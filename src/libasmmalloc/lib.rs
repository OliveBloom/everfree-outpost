//! A simple memory allocator designed for use in asm.js, where heap growth must be managed
//! externally.
//!
//! In an ordinary memory allocator, the allocator can detect OOM somewhere inside of malloc(),
//! request additional memory from the kernel, and then use the newly provided memory to complete
//! the pending malloc() call.  There was previously an asm.js extension to allow similar
//! functionality by permitting the asm.js code to swap out its heap array while running.  However,
//! this incurred a severe performance penalty under Chrome, and the feature was disabled by
//! default in Emscripten.
//!
//! This allocator works differently.  Instead of managing the heap size itself, asmmalloc provides
//! a mechanism for the external Javascript code to detect that the preallocated heap space is
//! close to running out, and relies on that external code to expand the heap when needed.
//! Crucially, because the heap expansion happens when no asm.js code is running, it can be done
//! with no specialized asm.js extensions: simply copy the heap contents into a new, larger
//! TypedArray, and re-instantiate the asm.js module with the larger array as its heap.
//!
//! The intended usage looks like this:
//!
//!     // Initialization:
//!     cur_heap = new Int32Array(1024 * 1024);
//!     asm_module = instantiate_asm_module(cur_heap);
//!
//!     // Normal operation:
//!     // ... run asm code ...
//!     if (top_of_heap() - max_allocated_address() < 256 * 1024) {
//!         old_heap = cur_heap
//!         cur_heap = new Int32Array(old_heap.length * 2);
//!         cur_heap.set(old_heap);
//!         asm_module = instantiate_asm_module(cur_heap);
//!     }
//!
//! This works as long as no single call to asm.js code increases heap usage by more than 256kB.
//! If any call does so, an allocation will fail during that call, as there is no way to increase
//! the heap size during asm.js execution.

#![feature(
    core_intrinsics,
    nonzero,
)]

// asmjs options
#![cfg_attr(asmjs, no_std)]
#![cfg_attr(asmjs, feature(allocator))]
#![cfg_attr(asmjs, allocator)]

// Non-asmjs options
#![cfg_attr(not(asmjs), feature(core))]

// asmjs imports
#[cfg(asmjs)] #[macro_use] extern crate asmrt;

// Non-asmjs imports
#[cfg(not(asmjs))] extern crate core;


use core::cmp;
use core::mem;
use core::nonzero::NonZero;
use core::ptr;


#[cfg(target_pointer_width = "32")]
const WORD_SIZE: usize = 4;
#[cfg(target_pointer_width = "32")]
const LOG_WORD_SIZE: usize = 2;

#[cfg(target_pointer_width = "64")]
const WORD_SIZE: usize = 8;
#[cfg(target_pointer_width = "64")]
const LOG_WORD_SIZE: usize = 3;

const HEADER_SIZE: usize = WORD_SIZE;
const BLOCK_ALIGN: usize = WORD_SIZE;

struct State {
    base_addr: usize,
    top_addr: usize,
}

const STATE_INIT: State = State {
    base_addr: 0,
    top_addr: 0,
};

const FLAG_IN_USE: usize = 0x01;
const FLAG_PREV_IN_USE: usize = 0x02;

struct BlockHeader {
    // Structure:
    //  - bit 0: FLAG_IN_USE
    //  - bit 1: FLAG_PREV_IN_USE
    //  - bit 2-3: log alignment (0..3 => LOG_WORD_SIZE .. LOG_WORD_SIZE + 3)
    //  - bit 4-31: size (in words)
    word: usize,
}

impl BlockHeader {
    fn in_use(&self) -> bool {
        (self.word & FLAG_IN_USE) != 0
    }

    fn prev_in_use(&self) -> bool {
        (self.word & FLAG_PREV_IN_USE) != 0
    }

    #[allow(dead_code)]
    fn alignment_bits(&self) -> usize {
        (self.word >> 2) & 3
    }

    #[allow(dead_code)]
    fn log_alignment(&self) -> usize {
        LOG_WORD_SIZE + self.alignment_bits()
    }

    #[allow(dead_code)]
    fn alignment(&self) -> usize {
        1 << self.log_alignment()
    }

    fn size(&self) -> usize {
        (self.word >> 4) << LOG_WORD_SIZE
    }

    fn set(&mut self, size: usize, align: usize, in_use: bool, prev_in_use: bool) {
        assert!(size % WORD_SIZE == 0,
                "size must a multiple of WORD_SIZE");

        let log_align = 
            if align != 0 {
                align.trailing_zeros() as usize
            } else {
                LOG_WORD_SIZE
            };
        assert!(LOG_WORD_SIZE <= log_align && log_align < LOG_WORD_SIZE + 4,
                "log_align out of range");

        self.word =
            ((size >> LOG_WORD_SIZE) << 4) |
            ((log_align - LOG_WORD_SIZE) << 2) |
            ((prev_in_use as usize) << 1) |
            ((in_use as usize) << 0);
    }

    fn set_prev_in_use(&mut self, prev_in_use: bool) {
        self.word = (self.word & !FLAG_PREV_IN_USE) | ((prev_in_use as usize) << 1);
    }
}


struct UsedBlock {
    hdr: BlockHeader,
}


struct FreeBlock {
    hdr: BlockHeader,
    prev: *mut FreeBlock,
    next: *mut FreeBlock,
}

/// Minimum block size.  Must be enough to fit a FreeBlock plus its boundary tag.
const MIN_BLOCK_SIZE: usize = 4 * WORD_SIZE;

impl FreeBlock {
    unsafe fn link(&mut self, prev: *mut FreeBlock, next: *mut FreeBlock) {
        (*prev).next = self as *mut _;
        (*next).prev = self as *mut _;
        self.next = next;
        self.prev = prev;
    }

    unsafe fn unlink(&mut self) {
        (*self.next).prev = self.prev;
        (*self.prev).next = self.next;
    }
}


#[inline]
fn aligned(val: usize, align: usize) -> usize {
    (val + align - 1) & !(align - 1)
}

impl State {
    unsafe fn init(&mut self, base_addr: usize, top_addr: usize) {
        self.base_addr = aligned(base_addr, BLOCK_ALIGN);
        // `self.top_addr` is actually set to the topmost *usable* address.  The additional space
        // between `self.top_addr` and `top_addr` is used to store the sentinel block.
        self.top_addr = (top_addr - mem::size_of::<FreeBlock>()) & !(BLOCK_ALIGN - 1);
        assert!(self.top_addr >= self.base_addr,
                "no heap space left after alignment");

        // Create sentinel block
        //
        // The sentinel block is a magical block that lives at the top end of memory.  It's a
        // member of the free list, but its header marks it as a used block so that the actual
        // topmost free block won't try to coalesce with it.  It also has size zero, and is missing
        // the usual boundary tag at the end.
        //
        // The sentinel block's PREV_IN_USE flag works like normal, which gives us a way to quickly
        // find the highest allocated address.

        let sent = self.sentinel();
        (*sent).hdr.set(0, 0, true, false);

        // Create main free block
        //
        // The lowest block (in memory order) always has PREV_IN_USE set, to prevent attempts to
        // coalesce backwards on free.
        let size = self.top_addr - self.base_addr;
        let base = self.base_addr;
        let free = self.create_free(base, size, true);

        (*free).link(sent, sent);
    }

    fn sentinel(&self) -> *mut FreeBlock {
        self.top_addr as *mut FreeBlock
    }

    unsafe fn iter_free(&self) -> FreeListIter {
        FreeListIter::new((*self.sentinel()).next)
    }

    unsafe fn iter_mem(&self) -> MemOrderIter {
        MemOrderIter::new(self.base_addr as *mut BlockHeader)
    }

    unsafe fn alloc(&mut self, obj_size: usize, align: usize) -> *mut u8 {
        let align = cmp::max(align, BLOCK_ALIGN);
        let obj_size = cmp::max(aligned(obj_size, align), MIN_BLOCK_SIZE - HEADER_SIZE);

        //println!("looking for {} bytes, {}-aligned", obj_size, align);

        for free in self.iter_free() {
            let free = *free;
            let free_size = (*free).hdr.size();
            // Early check
            if free_size < obj_size + HEADER_SIZE {
                continue;
            }
            if free == self.sentinel() {
                break;
            }

            let free_start = free as usize;
            let free_end = free_start + free_size;

            //println!("  candidate block: {} @ {:x}", free_size, free_start);

            // Find a start address for the object (NB: *not* for the UsedBlock).
            let mut obj_addr = aligned(free_start + HEADER_SIZE, align);
            if align > BLOCK_ALIGN {
                // Adjustment may be necessary.  The leftover space between free_start and
                // used_start must end up either being 0 or being >= MIN_BLOCK_SIZE.
                loop {
                    let offset = (obj_addr - HEADER_SIZE) - free_start;
                    if offset == 0 || offset >= MIN_BLOCK_SIZE {
                        break;
                    } else {
                        obj_addr += align;
                    }
                }
            }
            //println!("    place object @ {:x}", obj_addr);

            // Make sure the block will fit
            if obj_addr + HEADER_SIZE > free_end {
                continue;
            }

            // Compute the final UsedBlock size.  This may need to be larger than obj_size to avoid
            // leaving a FreeBlock of size < MIN_BLOCK_SIZE after the UsedBlock.
            let used_start = obj_addr - HEADER_SIZE;
            let mut used_size = HEADER_SIZE + obj_size;
            //println!("    used block begins at {:x}", used_start);

            let end_offset = free_end - (used_start + used_size);
            if end_offset != 0 && end_offset < MIN_BLOCK_SIZE {
                used_size += end_offset;
            }
            //println!("    used block size is {}", used_size);

            // Actually allocate the UsedBlock.
            self.alloc_block(free, used_start, used_size, align);
            println!("    allocated space!");
            return obj_addr as *mut u8;
        }

        // No free block was suitable - we're out of memory!
        ptr::null_mut()
    }

    unsafe fn free(&mut self, ptr: *mut u8) {
        let addr = ptr as usize;
        self.free_block((addr - HEADER_SIZE) as *mut UsedBlock);
    }

    unsafe fn get_highest_addr(&mut self) -> usize {
        let sent = self.sentinel();
        if (*sent).hdr.prev_in_use() {
            self.top_addr
        } else {
            let top_free_size = ptr::read((self.top_addr - WORD_SIZE) as *const usize);
            self.top_addr - top_free_size
        }
    }

    unsafe fn create_free(&mut self,
                          start: usize,
                          size: usize,
                          prev_in_use: bool) -> *mut FreeBlock {
        println!(" - create free block @{:x}, size {}, prev {}",
                 start - self.base_addr, size, prev_in_use);
        let ptr = start as *mut FreeBlock;
        (*ptr).hdr.set(size, 0, false, prev_in_use);
        ptr::write((start + size - WORD_SIZE) as *mut usize, size);
        ptr
    }

    unsafe fn create_used(&mut self,
                          start: usize,
                          size: usize,
                          align: usize,
                          prev_in_use: bool) -> *mut UsedBlock {
        println!(" - create used block @{:x}, size {}, align {}, prev {}",
                 start - self.base_addr, size, align, prev_in_use);
        let ptr = start as *mut UsedBlock;
        (*ptr).hdr.set(size, align, true, prev_in_use);
        ptr
    }

    unsafe fn alloc_block(&mut self,
                          free: *mut FreeBlock,
                          used_start: usize,
                          used_size: usize,
                          align: usize) {
        let used_end = used_start + used_size;

        let free_start = free as usize;
        let free_size = (*free).hdr.size();
        let free_end = free_start + free_size;

        // Cut a hole in the list, removing `free`.
        let mut cur = (*free).prev;
        let mut prev_in_use = (*free).hdr.prev_in_use();
        let after = (*free).next;

        // Data at `*free` may be invalidated after this point.

        // Create first free block, if needed.
        if free_start < used_start {
            let new = self.create_free(free_start, used_start - free_start, prev_in_use);
            // Partially link into the free list, following `cur`.
            (*cur).next = new;
            (*new).prev = cur;
            cur = new;
            prev_in_use = false;
        }

        // Create used block.
        self.create_used(used_start, used_size, align, prev_in_use);
        prev_in_use = true;

        // Create second free block, if needed.
        if used_end < free_end {
            let new = self.create_free(used_end, free_end - used_end, prev_in_use);
            (*cur).next = new;
            (*new).prev = cur;
            cur = new;
            prev_in_use = false;
        }

        // Finish linking blocks into list.
        (*cur).next = after;
        (*after).prev = cur;

        // Update prev_in_use for the following block.
        if prev_in_use {
            let mem_after_hdr = free_end as *mut BlockHeader;
            (*mem_after_hdr).set_prev_in_use(true);
        }
    }

    unsafe fn free_block(&mut self, used: *mut UsedBlock) {
        let mut start = used as usize;
        let mut end = start + (*used).hdr.size();
        let mut prev_in_use = (*used).hdr.prev_in_use();

        // The previous and next free blocks in list order.  If we coalesce with an adjacent block
        // (in memory order), the new block will take its place in the list.  Otherwise, it will be
        // inserted in an arbitrary location.
        let mut prev = ptr::null_mut();
        let mut next = ptr::null_mut();

        // Check the previous block in memory order, and try to coalesce.
        if !prev_in_use {
            println!(" + free: read prev size from {:x}", start - WORD_SIZE - self.base_addr);
            let free_size = ptr::read((start - WORD_SIZE) as *const usize);
            println!(" + free: coalesce backward ({} bytes)", free_size);
            let free = (start - free_size) as *mut FreeBlock;
            prev = (*free).prev;
            next = (*free).next;
            (*free).unlink();

            start -= free_size;
            prev_in_use = true;
        }

        // Check the next block in memory order.
        if !(*(end as *mut BlockHeader)).in_use() {
            let free = end as *mut FreeBlock;
            println!(" + free: coalesce forward ({} bytes)", (*free).hdr.size());
            prev = (*free).prev;
            next = (*free).next;
            (*free).unlink();

            end += (*free).hdr.size();
        }

        // Fallback setting for `prev` and `next`.
        if prev.is_null() {
            prev = self.sentinel();
            next = (*prev).next;
        }

        // We now know the span of the new free block, and its previous and next entries in the
        // free list.
        let new_free = self.create_free(start, end - start, prev_in_use);
        (*new_free).link(prev, next);

        // Update prev_in_use for the following block.
        (*(end as *mut BlockHeader)).set_prev_in_use(false);
    }


    unsafe fn debug_print(&mut self) {
        for hdr in self.iter_mem() {
            let hdr = *hdr;

            let addr = hdr as usize - self.base_addr;
            let size = (*hdr).size();
            let is_sentinel = hdr as usize == self.sentinel() as usize;
            let code =
                if is_sentinel { "S" }
                else if (*hdr).in_use() { "X" }
                else { "_" };

            if (*hdr).in_use() {
                println!("  block [{}]: {:8x} - {:8x}, size = {} (word = {:x})",
                         code, addr, addr + size, size, (*hdr).word);
            } else {
                let free = hdr as *mut FreeBlock;
                println!("  block [{}]: {:8x} - {:8x}, size = {} (list: {:x} <-> {:x} <-> {:x})",
                         code, addr, addr + size, size,
                         (*free).prev as usize - self.base_addr,
                         free as usize - self.base_addr,
                         (*free).next as usize - self.base_addr);
            }

            if is_sentinel {
                break;
            }
        }

        println!("Top addr: {:x}", self.get_highest_addr() - self.base_addr);
    }
}


struct FreeListIter {
    ptr: *mut FreeBlock,
}

impl FreeListIter {
    unsafe fn new(ptr: *mut FreeBlock) -> FreeListIter {
        FreeListIter { ptr: ptr }
    }
}

impl Iterator for FreeListIter {
    type Item = NonZero<*mut FreeBlock>;

    fn next(&mut self) -> Option<NonZero<*mut FreeBlock>> {
        if self.ptr.is_null() {
            None
        } else {
            unsafe {
                let result = self.ptr;
                self.ptr = (*self.ptr).next;
                Some(NonZero::new(result))
            }
        }
    }
}


struct MemOrderIter {
    ptr: *mut BlockHeader,
}

impl MemOrderIter {
    unsafe fn new(ptr: *mut BlockHeader) -> MemOrderIter {
        MemOrderIter { ptr: ptr }
    }
}

impl Iterator for MemOrderIter {
    type Item = NonZero<*mut BlockHeader>;

    fn next(&mut self) -> Option<NonZero<*mut BlockHeader>> {
        unsafe {
            let result = self.ptr;
            let old_addr = self.ptr as usize;
            let new_addr = old_addr + (*self.ptr).size();
            self.ptr = new_addr as *mut BlockHeader;
            Some(NonZero::new(result))
        }
    }
}


#[cfg(asmjs)]
pub mod __allocator {
    use core::intrinsics;
    use core::ptr;

    use super::{State, STATE_INIT};

    static mut STATE: State = STATE_INIT;

    // Public asmmalloc_* API
    #[no_mangle]
    pub unsafe extern "C" fn asmmalloc_init(base: *mut u8,
                                            top: *mut u8) {
        STATE.init(base as usize, top as usize);
    }

    #[no_mangle]
    pub unsafe extern "C" fn asmmalloc_reinit(_base: *mut u8,
                                              _top: *mut u8) {
        assert!(false, "reinit not yet implemented");
    }

    #[no_mangle]
    pub unsafe extern "C" fn asmmalloc_max_allocated_address() -> usize {
        STATE.get_highest_addr()
    }

    #[no_mangle]
    pub unsafe extern "C" fn asmmalloc_alloc(size: usize, align: usize) -> *mut u8 {
        STATE.alloc(size, align)
    }

    #[no_mangle]
    pub unsafe extern "C" fn asmmalloc_free(ptr: *mut u8) {
        STATE.free(ptr)
    }

    #[no_mangle]
    pub unsafe extern "C" fn asmmalloc_debug_print() {
        STATE.debug_print()
    }


    // Internal __rust_* API, used by Rust's liballoc
    #[no_mangle]
    pub unsafe extern "C" fn __rust_allocate(size: usize,
                                             align: usize) -> *mut u8 {
        STATE.alloc(size, align)
    }

    #[no_mangle]
    pub unsafe extern "C" fn __rust_reallocate(ptr: *mut u8,
                                               old_size: usize,
                                               size: usize,
                                               align: usize) -> *mut u8 {
        let new = STATE.alloc(size, align);
        if new.is_null() {
            return ptr::null_mut();
        }

        intrinsics::copy_nonoverlapping(ptr, new, old_size);
        STATE.free(ptr);
        new
    }

    #[no_mangle]
    pub unsafe extern "C" fn __rust_reallocate_inplace(_ptr: *mut u8,
                                                       old_size: usize,
                                                       _size: usize,
                                                       _align: usize) -> usize {
        old_size
    }

    #[no_mangle]
    pub unsafe extern "C" fn __rust_deallocate(ptr: *mut u8,
                                               _old_size: usize,
                                               _align: usize) {
        STATE.free(ptr)
    }

    #[no_mangle]
    pub unsafe extern "C" fn __rust_usable_size(size: usize,
                                                _align: usize) -> usize {
        size
    }
}



#[cfg(not(asmjs))]
#[allow(unused_variables)]
fn main() {
    let buf = Box::new([0u8; 1024 * 1024]);
    let mut state = STATE_INIT;
    unsafe {
        let addr = buf.as_ptr() as usize;
        state.init(addr, addr + buf.len());
        let a = state.alloc(16, 4);
        let b = state.alloc(16, 2);
        let c = state.alloc(16, 8);
        let d = state.alloc(16, 8);
        let e = state.alloc(32, 32);
        state.debug_print();
        state.free(a);
        state.free(c);
        state.free(e);
        let e = state.alloc(1048384, 4);
        state.free(e);
        state.free(b);
        state.free(d);
        state.debug_print(); return;

    }
}
