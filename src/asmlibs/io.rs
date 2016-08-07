use std::cmp;
use std::io;

pub struct Cursor<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> Cursor<'a> {
    pub fn new(buf: &'a [u8]) -> Cursor<'a> {
        Cursor {
            buf: buf,
            pos: 0,
        }
    }
}

impl<'a> io::Read for Cursor<'a> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let amt = cmp::min(self.buf.len() - self.pos, buf.len());
        let start = self.pos;

        let src = &self.buf[start .. start + amt];
        let dest = &mut buf[0 .. amt];
        dest.copy_from_slice(src);

        self.pos += amt;
        Ok(amt)
    }
}
