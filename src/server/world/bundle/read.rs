use std::io;
use std::iter;
use std::mem;
use std::slice;

use types::*;
use util::Bytes;
use util::Convert;

use world::extra;
use world::Item;
use world::Motion;
use world::{EntityAttachment, InventoryAttachment, StructureAttachment};
use world::{TerrainChunkFlags, StructureFlags};

use super::types::*;
use super::padding;
pub use super::error::Result as Result;
pub use super::error::Error as Error;


pub trait ReadExt: io::Read {
    // TODO: remove after updating to a Rust version with read_exact in std::io::Read
    fn read_exact(&mut self, buf: &mut [u8]) -> Result<()> {
        let mut base = 0;
        while base < buf.len() {
            let n = try!(self.read(&mut buf[base..]));
            assert!(n > 0 && base + n <= buf.len());
            base += n;
        }
        Ok(())
    }

    fn read_bytes(&mut self, len: usize) -> Result<Vec<u8>> {
        let pad = padding(len);
        let mut vec = iter::repeat(0_u8).take(len + pad).collect::<Vec<_>>();
        try!(self.read_exact(&mut vec));
        vec.truncate(len);
        Ok(vec)
    }

    fn read_str_bytes(&mut self, len: usize) -> Result<Box<str>> {
        match String::from_utf8(try!(self.read_bytes(len))) {
            Ok(s) => Ok(s.into_boxed_slice()),
            Err(_) => fail!("utf8 encoding error"),
        }
    }

    fn read_str(&mut self) -> Result<Box<str>> {
        let len = try!(self.read_count());
        self.read_str_bytes(len)
    }

    fn read_val<T: Bytes>(&mut self) -> Result<T> {
        let len = mem::size_of::<T>();
        let pad = padding(len);

        // `u32` is enough space for the padding (3 bytes max)
        let mut result: (T, u32) = unsafe { mem::zeroed() };
        assert!(mem::size_of_val(&result) >= len + pad);
        let buf = unsafe {
            slice::from_raw_parts_mut(&mut result as *mut (T, u32) as *mut u8, len + pad)
        };
        try!(self.read_exact(buf));
        Ok(result.0)
    }

    fn read_count(&mut self) -> Result<usize> {
        let count = try!(self.read_val::<u32>());
        Ok(unwrap!(count.to_usize()))
    }

    fn read_slice<T, B, F>(&mut self, mut f: F) -> Result<Box<[T]>>
            where B: Bytes, F: FnMut(B) -> Result<T> {
        let count = try!(self.read_count());
        let mut v = Vec::with_capacity(count);
        for _ in 0 .. count {
            let x = try!(self.read_val());
            v.push(try!(f(x)));
        }
        Ok(v.into_boxed_slice())
    }
}

impl<R: io::Read> ReadExt for R {}


fn read_entity_attachment<R: io::Read>(r: &mut R) -> Result<EntityAttachment> {
    let (tag, a, b): (u8, u8, u16) = try!(r.read_val());
    match tag {
        1 => Ok(EntityAttachment::World),
        2 => Ok(EntityAttachment::Chunk),
        3 => Ok(EntityAttachment::Client(ClientId(b))),
        _ => fail!("bad tag for EntityAttachment"),
    }
}

fn read_inventory_attachment<R: io::Read>(r: &mut R) -> Result<InventoryAttachment> {
    let (tag, a, b): (u8, u8, u16) = try!(r.read_val());
    match tag {
        1 => Ok(InventoryAttachment::World),
        2 => Ok(InventoryAttachment::Client(ClientId(b))),
        3 => {
            let id: u32 = try!(r.read_val());
            Ok(InventoryAttachment::Entity(EntityId(id)))
        },
        4 => {
            let id: u32 = try!(r.read_val());
            Ok(InventoryAttachment::Structure(StructureId(id)))
        },
        _ => fail!("bad tag for InventoryAttachment"),
    }
}

fn read_structure_attachment<R: io::Read>(r: &mut R) -> Result<StructureAttachment> {
    let (tag, a, b): (u8, u8, u16) = try!(r.read_val());
    match tag {
        1 => Ok(StructureAttachment::Plane),
        2 => Ok(StructureAttachment::Chunk),
        _ => fail!("bad tag for StructureAttachment"),
    }
}


pub fn read_client<R: io::Read>(r: &mut R) -> Result<Client> {
    let name = try!(r.read_str());
    let pawn = match try!(r.read_val()) {
        -1 => None,
        id => Some(EntityId(id)),
    };

    let stable_id = try!(r.read_val());
    let child_entities = try!(r.read_slice(|raw| Ok(EntityId(raw))));
    let child_inventories = try!(r.read_slice(|raw| Ok(InventoryId(raw))));

    Ok(Client {
        name: name,
        pawn: pawn,

        stable_id: stable_id,
        child_entities: child_entities,
        child_inventories: child_inventories,
    })
}

pub fn read_entity<R: io::Read>(r: &mut R) -> Result<Entity> {
    let stable_plane = Stable::new(try!(r.read_val()));

    let (start_time, duration, start_pos, end_pos) = try!(r.read_val());
    let (anim, facing, target_velocity, appearance) = try!(r.read_val());

    let extra = try!(extra::read(r));
    let stable_id = try!(r.read_val());
    let attachment = try!(read_entity_attachment(r));
    let child_inventories = try!(r.read_slice(|raw| Ok(InventoryId(raw))));

    Ok(Entity {
        stable_plane: stable_plane,

        motion: Motion {
            start_time: start_time,
            duration: duration,
            start_pos: start_pos,
            end_pos: end_pos,
        },
        anim: anim,
        facing: facing,
        target_velocity: target_velocity,
        appearance: appearance,

        extra: extra,
        stable_id: stable_id,
        attachment: attachment,
        child_inventories: child_inventories,
    })
}

pub fn read_inventory<R: io::Read>(r: &mut R) -> Result<Inventory> {
    let contents = try!(r.read_slice(|(tag, a, b): (u8, u8, u16)| {
        match tag {
            1 => Ok(Item::Empty),
            2 => Ok(Item::Bulk(a, b)),
            3 => Ok(Item::Special(a, b)),
            _ => fail!("bad tag for Item"),
        }
    }));

    let stable_id = try!(r.read_val());
    let attachment = try!(read_inventory_attachment(r));

    Ok(Inventory {
        contents: contents,

        stable_id: stable_id,
        attachment: attachment,
    })
}

pub fn read_plane<R: io::Read>(r: &mut R) -> Result<Plane> {
    let name = try!(r.read_str());

    let saved_chunks = try!(r.read_slice(|(cpos, pid)| Ok((cpos, Stable::new(pid)))));

    let stable_id = try!(r.read_val());

    Ok(Plane {
        name: name,

        saved_chunks: saved_chunks,

        stable_id: stable_id,
    })
}

pub fn read_terrain_chunk<R: io::Read>(r: &mut R) -> Result<TerrainChunk> {
    let (raw_stable_plane, cpos) = try!(r.read_val());
    let stable_plane = Stable::new(raw_stable_plane);
    let raw_blocks = try!(r.read_slice(|x| Ok(x)));

    if raw_blocks.len() != CHUNK_TOTAL {
        fail!("wrong number of blocks in chunk");
    }
    let mut blocks = Box::new(EMPTY_CHUNK);
    for (src, dst) in raw_blocks.iter().zip(blocks.iter_mut()) {
        *dst = *src;
    }
    let blocks = blocks;

    let stable_id = try!(r.read_val());
    let flags = unwrap!(TerrainChunkFlags::from_bits(try!(r.read_val())));
    let child_structures = try!(r.read_slice(|raw| Ok(StructureId(raw))));

    Ok(TerrainChunk {
        stable_plane: stable_plane,
        cpos: cpos,
        blocks: blocks,

        stable_id: stable_id,
        flags: flags,
        child_structures: child_structures,
    })
}

pub fn read_structure<R: io::Read>(r: &mut R) -> Result<Structure> {
    let (raw_stable_plane, pos, template) = try!(r.read_val());
    let stable_plane = Stable::new(raw_stable_plane);

    let stable_id = try!(r.read_val());
    let flags = unwrap!(StructureFlags::from_bits(try!(r.read_val())));
    let attachment = try!(read_structure_attachment(r));
    let child_inventories = try!(r.read_slice(|raw| Ok(InventoryId(raw))));

    Ok(Structure {
        stable_plane: stable_plane,
        pos: pos,
        template: template,

        stable_id: stable_id,
        flags: flags,
        attachment: attachment,
        child_inventories: child_inventories,
    })
}


fn read_multi<R, T, F>(r: &mut R, len: usize, mut f: F) -> Result<Box<[T]>>
        where R: io::Read, F: FnMut(&mut R) -> Result<T> {
    let mut v = Vec::with_capacity(len);
    for _ in 0 .. len {
        v.push(try!(f(r)));
    }
    Ok(v.into_boxed_slice())
}

pub fn read<R: io::Read>(r: &mut R) -> Result<Bundle> {
    let (anims_len, items_len, blocks_len, templates_len,
         clients_len, entities_len, inventories_len,
         planes_len, terrain_chunks_len, structures_len) = try!(r.read_val());

    let anims = try!(read_multi(r, anims_len, |r| r.read_str()));
    let items = try!(read_multi(r, items_len, |r| r.read_str()));
    let blocks = try!(read_multi(r, blocks_len, |r| r.read_str()));
    let templates = try!(read_multi(r, templates_len, |r| r.read_str()));

    let clients = try!(read_multi(r, clients_len, read_client));
    let entities = try!(read_multi(r, entities_len, read_entity));
    let inventories = try!(read_multi(r, inventories_len, read_inventory));
    let planes = try!(read_multi(r, planes_len, read_plane));
    let terrain_chunks = try!(read_multi(r, terrain_chunks_len, read_terrain_chunk));
    let structures = try!(read_multi(r, structures_len, read_structure));

    Ok(Bundle {
        anims: anims,
        items: items,
        blocks: blocks,
        templates: templates,

        clients: clients,
        entities: entities,
        inventories: inventories,
        planes: planes,
        terrain_chunks: terrain_chunks,
        structures: structures,
    })
}
