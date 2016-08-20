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


const BUF_SIZE: usize = 16;
pub const DISPLAY_TIME: Time = 5000;

// A fixed-size buffer, in which some slots in the "full" part of the buffer may be logically
// empty.  The buffer can be compacted to free up capacity.  In addition to the standard `len`
// field, the buffer has an `in_use` count to make it easy to determine whether it's possible to
// compact.
pub struct InvChanges {
    buf: [Entry; BUF_SIZE],
    /// Offset of one-past-the-last slot.
    len: usize,
    /// Number of slots in the range `0 .. len` that contain nonzero entries.  (Slots outside the
    /// range always contain zero entries.)
    in_use: usize,
}

impl InvChanges {
    pub fn new() -> InvChanges {
        InvChanges {
            buf: [Entry::empty(); BUF_SIZE],
            len: 0,
            in_use: 0,
        }
    }

    fn compact(&mut self) {
        if self.len == self.in_use {
            return;
        }

        let mut read = 0;
        let mut write = 0;

        while read < self.len {
            if self.buf[read].item != 0 {
                self.buf[write] = self.buf[read];
                read += 1;
                write += 1;
            } else {
                read += 1;
            }
        }
        assert!(write == self.in_use);
        while write < self.len {
            self.buf[write] = Entry::empty();
            write += 1;
        }

        self.len = self.in_use;
    }

    pub fn add(&mut self, now: Time, item: ItemId, count: i16) {
        if item == NO_ITEM {
            return;
        }

        // First, collect any existing entries for this item.
        let mut count = count;
        for entry in &mut self.buf {
            if entry.item != NO_ITEM && entry.time < now - DISPLAY_TIME {
                // Entry is expired, clear it out.
                *entry = Entry::empty();
                self.in_use -= 1;
            } else if entry.item == item {
                // Entry isn't expired, but contains the right item.  Merge it into the new entry.
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
            // Buffer is completely full.  Remove the oldest entry.
            self.buf[0] = Entry::empty();
            self.in_use -= 1;
        }
        if self.len == BUF_SIZE {
            self.compact();
        }

        self.buf[self.len] = Entry::new(item, count, now);
        self.len += 1;
        self.in_use += 1;
    }

    pub fn clear(&mut self) {
        for entry in &mut self.buf {
            *entry = Entry::empty();
        }
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
            let entry = &self.owner.buf[self.index];
            self.index += 1;
            if entry.item != NO_ITEM {
                return Some(entry);
            }
        }
        None
    }
}
