use types::*;


#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Entry {
    pub item: ItemId,
    pub count: i16,
    pub time: Time,
}

impl Entry {
    fn new(item: ItemId, count: i16, time: Time) -> Entry {
        Entry {
            item: item,
            count: count,
            time: time,
        }
    }

    fn empty() -> Entry {
        Entry::new(NO_ITEM, 0, 0)
    }
}


const BUF_SIZE: usize = 32;
pub const DISPLAY_TIME: Time = 5000;

// A fixed-size ringbuffer, in which some slots in the "full" part of the buffer may be logically
// empty.  The buffer can be compacted to free up space.  In addition to the standard `start` and
// `len` fields, the buffer has an `in_use` count to make it easy to determine whether it's
// possible to compact.
pub struct InvChanges {
    buf: [Entry; BUF_SIZE],
    /// Index of the first slot that's in use.
    start: usize,
    /// Offset of one-past-the-last slot from the first.
    len: usize,
    /// Number of slots in the range `start .. start + len` that contain nonzero entries.  (Slots
    /// outside the range always contain zero entries.)
    in_use: usize,
}

impl InvChanges {
    pub fn new() -> InvChanges {
        InvChanges {
            buf: [Entry::empty(); BUF_SIZE],
            start: 0,
            len: 0,
            in_use: 0,
        }
    }

    #[inline(always)]
    fn get(&self, off: usize) -> Entry {
        self.buf[(self.start + off) % BUF_SIZE]
    }

    #[inline(always)]
    fn get_ref(&self, off: usize) -> &Entry {
        &self.buf[(self.start + off) % BUF_SIZE]
    }

    #[inline(always)]
    fn get_mut(&mut self, off: usize) -> &mut Entry {
        &mut self.buf[(self.start + off) % BUF_SIZE]
    }

    fn compact(&mut self) {
        if self.len == self.in_use {
            return;
        }

        let mut read = 0;
        let mut write = 0;

        while read < self.len {
            if self.get(read).item != 0 {
                *self.get_mut(write) = self.get(read);
                read += 1;
                write += 1;
            } else {
                read += 1;
            }
        }
        assert!(write == self.in_use);
        self.len = write;
    }

    pub fn add(&mut self, now: Time, item: ItemId, count: i16) {
        if item == NO_ITEM {
            return;
        }

        // First, collect any existing entries for this item.
        let mut count = count;
        for entry in &mut self.buf {
            if entry.item != NO_ITEM && entry.time < now - DISPLAY_TIME {
                *entry = Entry::empty();
                self.in_use -= 1;
            } else if entry.item == item {
                count += entry.count;
                *entry = Entry::empty();
                self.in_use -= 1;
            }
        }

        if count == 0 {
            return;
        }

        // Now add the new entry.
        if self.in_use == BUF_SIZE {
            // All entries are in use.  Replace the oldest item with this one.
            let idx = self.len;
            *self.get_mut(idx) = Entry::new(item, count, now);
            self.start = (self.start + 1) % BUF_SIZE;
        } else {
            // Add the new entry to the end of the list.
            if self.len == BUF_SIZE {
                self.compact();
            }
            let idx = self.len;
            *self.get_mut(idx) = Entry::new(item, count, now);
            self.len += 1;
            self.in_use += 1;
        }
    }

    pub fn clear(&mut self) {
        for entry in &mut self.buf {
            *entry = Entry::empty();
        }
        self.start = 0;
        self.len = 0;
        self.in_use = 0;
    }

    pub fn iter(&self) -> Iter {
        Iter {
            owner: self,
            index: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.in_use
    }
}

pub struct Iter<'a> {
    owner: &'a InvChanges,
    index: usize,
}

impl<'a> Iterator for Iter<'a> {
    type Item = &'a Entry;

    fn next(&mut self) -> Option<&'a Entry> {
        while self.index < self.owner.len {
            let entry = self.owner.get_ref(self.index);
            self.index += 1;
            if entry.item != NO_ITEM {
                return Some(entry);
            }
        }
        None
    }
}
