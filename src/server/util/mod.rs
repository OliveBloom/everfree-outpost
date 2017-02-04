use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::isize;
use std::marker::Unsize;
use std::mem;
use std::panic;
use std::ptr;
use time;

use types::Time;

pub use libcommon_util::*;

pub use self::cursor::Cursor;
pub use self::id_map::IdMap;
pub use self::refcount::RefcountedMap;
pub use self::stable_id_map::{StableIdMap, IntrusiveStableId};

pub mod cursor;
pub mod id_map;
pub mod refcount;
#[macro_use] pub mod stable_id_map;


pub fn now() -> Time {
    let timespec = time::get_time();
    (timespec.sec as Time * 1000) + (timespec.nsec / 1000000) as Time
}


pub fn multimap_insert<K, V>(map: &mut HashMap<K, HashSet<V>>, k: K, v: V)
        where K: Hash+Eq,
              V: Hash+Eq {
    use std::collections::hash_map::Entry::*;
    let bucket = match map.entry(k) {
        Vacant(e) => e.insert(HashSet::new()),
        Occupied(e) => e.into_mut(),
    };
    bucket.insert(v);
}

pub fn multimap_remove<K, V>(map: &mut HashMap<K, HashSet<V>>, k: K, v: V)
        where K: Hash+Eq,
              V: Hash+Eq {
    use std::collections::hash_map::Entry::*;
    match map.entry(k) {
        Vacant(_) => { },
        Occupied(mut e) => {
            e.get_mut().remove(&v);
            if e.get().is_empty() {
                e.remove();
            }
        },
    }
}


pub struct OptionIter<I>(Option<I>);

impl<I: Iterator> Iterator for OptionIter<I> {
    type Item = <I as Iterator>::Item;

    fn next(&mut self) -> Option<<I as Iterator>::Item> {
        match self.0 {
            Some(ref mut iter) => iter.next(),
            None => None,
        }
    }
}

pub trait OptionIterExt<I> {
    fn unwrap_iter(self) -> OptionIter<I>;
}

impl<I: Iterator> OptionIterExt<I> for Option<I> {
    fn unwrap_iter(self) -> OptionIter<I> {
        OptionIter(self)
    }
}


pub fn encode_rle16<I: Iterator<Item=u16>>(iter: I) -> Vec<u16> {
    let mut result = Vec::new();

    let mut iter = iter.peekable();
    while !iter.peek().is_none() {
        let cur = iter.next().unwrap();

        // TODO: check that count doesn't overflow 12 bits.
        let mut count = 1u16;
        while count < 0x0fff && iter.peek().map(|&x| x) == Some(cur) {
            iter.next();
            count += 1;
        }
        if count > 1 {
            result.push(0xf000 | count);
        }
        result.push(cur);
    }

    result
}


/// Build a fixed-length array, filling each slot with the result of `f()`.  `A` must be a
/// fixed-length array type, such as `[T; 10]`.
///
/// The `A: Unsize<[T]>` bound imposes roughly the desired constraint, but I'm not sure it's
/// exactly right, so this function is unsafe for now.
pub unsafe fn fixed_array_with<T, A, F>(mut f: F) -> A
        where F: FnMut() -> T,
              A: Unsize<[T]> {
    let mut arr: A = mem::zeroed();
    let ptr = &mut arr as *mut A as *mut T;

    // Need some special handling to avoid running `drop` on uninitialized array elements, in the
    // event of a panic inside f().
    let res = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        let len = mem::size_of::<A>() / mem::size_of::<T>();
        assert!(len <= isize::MAX as usize);
        for i in 0 .. len as isize {
            ptr::write(ptr.offset(i), f());
        }
    }));

    match res {
        Ok(()) => arr,
        Err(e) => {
            mem::forget(arr);
            panic::resume_unwind(e)
        },
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_fixed_array_with() {
        use super::fixed_array_with;

        let a: [u8; 10] = unsafe { fixed_array_with(|| 99) };
        assert_eq!(a, [99; 10]);

        let b: [Vec<u8>; 10] = unsafe { fixed_array_with(Vec::new) };
        assert!(b.iter().all(|v| v.len() == 0));
        assert!(b.iter().all(|v| v.capacity() == 0));

        let c: [Vec<u8>; 10] = unsafe { fixed_array_with(|| Vec::with_capacity(3)) };
        assert!(c.iter().all(|v| v.len() == 0));
        assert!(c.iter().all(|v| v.capacity() >= 3));
    }
}
