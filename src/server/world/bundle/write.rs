use std::io;
use std::mem;
use std::slice;

use types::*;
use util::Bytes;
use util::Convert;

use world::extra;
use world::Item;
use world::{EntityAttachment, InventoryAttachment, StructureAttachment};

use super::types::*;
use super::padding;
pub use super::error::Result as Result;
pub use super::error::Error as Error;

pub trait WriteExt: io::Write {
    fn write_bytes(&mut self, buf: &[u8]) -> Result<()> {
        try!(self.write_all(buf));
        let pad = padding(buf.len());
        if pad > 0 {
            try!(self.write_all(&[0; 3][..pad]));
        }
        Ok(())
    }

    fn write_str_bytes(&mut self, s: &str) -> Result<()> {
        self.write_bytes(s.as_bytes())
    }

    fn write_str(&mut self, s: &str) -> Result<()> {
        try!(self.write_count(s.len()));
        self.write_str_bytes(s)
    }

    fn write_val<T: Bytes>(&mut self, x: T) -> Result<()> {
        let len = mem::size_of::<T>();
        let buf = unsafe {
            slice::from_raw_parts(&x as *const T as *const u8, len)
        };
        try!(self.write_bytes(buf));
        Ok(())
    }

    fn write_count(&mut self, count: usize) -> Result<()> {
        try!(self.write_val::<u32>(unwrap!(count.to_u32())));
        Ok(())
    }

    fn write_slice<T, B, F>(&mut self, xs: &[T], mut f: F) -> Result<()>
            where B: Bytes, F: FnMut(&T) -> B {
        try!(self.write_count(xs.len()));
        for x in xs {
            try!(self.write_val(f(x)));
        }
        Ok(())
    }
}

impl<W: io::Write> WriteExt for W {}


fn write_entity_attachment<W: io::Write>(w: &mut W, a: EntityAttachment) -> Result<()> {
    match a {
        EntityAttachment::World => w.write_val(1u8),
        EntityAttachment::Chunk => w.write_val(2u8),
        // Pack the id with the tag, since they both fit in under 4 bytes.
        EntityAttachment::Client(id) => w.write_val((3u8, 0u8, id.unwrap())),
    }
}

fn write_inventory_attachment<W: io::Write>(w: &mut W, a: InventoryAttachment) -> Result<()> {
    match a {
        InventoryAttachment::World => w.write_val(1u8),
        InventoryAttachment::Client(id) => w.write_val((2u8, 0u8, id.unwrap())),
        // ID doesn't fit with tag, so put it after.
        InventoryAttachment::Entity(id) => w.write_val((3u8, 0u8, 0u16, id.unwrap())),
        InventoryAttachment::Structure(id) => w.write_val((4u8, 0u8, 0u16, id.unwrap())),
    }
}

fn write_structure_attachment<W: io::Write>(w: &mut W, a: StructureAttachment) -> Result<()> {
    match a {
        StructureAttachment::Plane => w.write_val(1u8),
        StructureAttachment::Chunk => w.write_val(2u8),
    }
}


pub fn write_client<W: io::Write>(w: &mut W, c: &Client) -> Result<()> {
    try!(w.write_str(&c.name));
    try!(w.write_val(c.pawn.map_or(-1_i32 as u32, |eid| eid.unwrap())));

    try!(w.write_val(c.stable_id));
    try!(w.write_slice(&c.child_entities, |eid| eid.unwrap()));
    try!(w.write_slice(&c.child_inventories, |iid| iid.unwrap()));

    Ok(())
}

pub fn write_entity<W: io::Write>(w: &mut W, e: &Entity) -> Result<()> {
    try!(w.write_val(e.stable_plane.unwrap()));

    try!(w.write_val((e.motion.start_time,
                      e.motion.duration,
                      e.motion.start_pos,
                      e.motion.end_pos)));
    try!(w.write_val((e.anim,
                      e.facing,
                      e.target_velocity,
                      e.appearance)));

    try!(extra::write(w, &e.extra));
    try!(w.write_val(e.stable_id));
    try!(write_entity_attachment(w, e.attachment));
    try!(w.write_slice(&e.child_inventories, |iid| iid.unwrap()));

    Ok(())
}

pub fn write_inventory<W: io::Write>(w: &mut W, i: &Inventory) -> Result<()> {
    try!(w.write_slice(&i.contents, |item| -> (u8, u8, u16) {
        match *item {
            Item::Empty => (1, 0, 0),
            Item::Bulk(count, id) => (2, count, id),
            Item::Special(idx, id) => (3, idx, id),
        }
    }));

    try!(w.write_val(i.stable_id));
    try!(write_inventory_attachment(w, i.attachment));

    Ok(())
}

pub fn write_plane<W: io::Write>(w: &mut W, p: &Plane) -> Result<()> {
    try!(w.write_str(&p.name));

    try!(w.write_slice(&p.saved_chunks, |&(cpos, pid)| (cpos, pid.unwrap())));

    try!(w.write_val(p.stable_id));

    Ok(())
}

pub fn write_terrain_chunk<W: io::Write>(w: &mut W, tc: &TerrainChunk) -> Result<()> {
    try!(w.write_val((tc.stable_plane.unwrap(),
                      tc.cpos)));
    // This way of doing tc.blocks might be slow, but maybe LLVM can handle it?
    try!(w.write_slice(&*tc.blocks, |&x| x));

    try!(w.write_val(tc.stable_id));
    try!(w.write_val(tc.flags.bits()));
    try!(w.write_slice(&tc.child_structures, |sid| sid.unwrap()));

    Ok(())
}

pub fn write_structure<W: io::Write>(w: &mut W, s: &Structure) -> Result<()> {
    try!(w.write_val((s.stable_plane.unwrap(),
                      s.pos,
                      s.template)));

    try!(w.write_val(s.stable_id));
    try!(w.write_val(s.flags.bits()));
    try!(write_structure_attachment(w, s.attachment));
    try!(w.write_slice(&s.child_inventories, |iid| iid.unwrap()));

    Ok(())
}

pub fn write_bundle<W: io::Write>(w: &mut W, b: &Bundle) -> Result<()> {
    try!(w.write_count(b.anims.len()));
    try!(w.write_count(b.items.len()));
    try!(w.write_count(b.blocks.len()));
    try!(w.write_count(b.templates.len()));

    try!(w.write_count(b.clients.len()));
    try!(w.write_count(b.entities.len()));
    try!(w.write_count(b.inventories.len()));
    try!(w.write_count(b.planes.len()));
    try!(w.write_count(b.terrain_chunks.len()));
    try!(w.write_count(b.structures.len()));

    for a in b.anims.iter() {
        try!(w.write_str(a));
    }
    for i in b.items.iter() {
        try!(w.write_str(i));
    }
    for b in b.blocks.iter() {
        try!(w.write_str(b));
    }
    for t in b.templates.iter() {
        try!(w.write_str(t));
    }

    for c in b.clients.iter() {
        try!(write_client(w, c));
    }
    for e in b.entities.iter() {
        try!(write_entity(w, e));
    }
    for i in b.inventories.iter() {
        try!(write_inventory(w, i));
    }
    for p in b.planes.iter() {
        try!(write_plane(w, p));
    }
    for tc in b.terrain_chunks.iter() {
        try!(write_terrain_chunk(w, tc));
    }
    for s in b.structures.iter() {
        try!(write_structure(w, s));
    }

    Ok(())
}
