use std::prelude::v1::*;
use std::ops::{Deref, DerefMut};
use std::ptr;

use physics::v3::{V3, V2, scalar, Region, RegionPoints};
use physics::CHUNK_SIZE;
use ATLAS_SIZE;
use LOCAL_SIZE;

use IntrusiveCorner;
use {emit_quad, remaining_quads};
use types::{BlockData, LocalChunks};


/// Vertex attributes for terrain.
#[allow(dead_code)]
#[derive(Clone, Copy)]
pub struct Vertex {
    corner: (u8, u8),
    pos: (u8, u8, u8),
    side: u8,
    tex_coord: (u8, u8),
}

impl IntrusiveCorner for Vertex {
    fn corner(&self) -> &(u8, u8) { &self.corner }
    fn corner_mut(&mut self) -> &mut (u8, u8) { &mut self.corner }
}


pub struct GeomGenState {
    cpos: V2,
    iter: RegionPoints<V3>,
}

impl GeomGenState {
    pub fn new(cpos: V2) -> GeomGenState {
        GeomGenState {
            cpos: cpos,
            iter: Region::new(scalar(0), scalar::<V3>(CHUNK_SIZE)).points(),
        }
    }
}

pub struct GeomGen<'a> {
    local_chunks: &'a LocalChunks,
    block_data: &'a [BlockData],
    state: &'a mut GeomGenState,
}

impl<'a> GeomGen<'a> {
    pub fn new(local_chunks: &'a LocalChunks,
               block_data: &'a [BlockData],
               state: &'a mut GeomGenState) -> GeomGen<'a> {
        GeomGen {
            local_chunks: local_chunks,
            block_data: block_data,
            state: state,
        }
    }

    pub fn generate(&mut self,
                    buf: &mut [Vertex],
                    idx: &mut usize) -> bool {
        let local_bounds = Region::new(scalar(0), scalar(LOCAL_SIZE as i32));
        let chunk_idx = local_bounds.index(self.cpos);
        let chunk_bounds = Region::new(scalar(0), scalar(CHUNK_SIZE));

        while remaining_quads(buf, *idx) >= 4 {
            let iter_pos = match self.iter.next() {
                Some(p) => p,
                None => return false,
            };

            let pos = self.cpos.extend(0) * scalar(CHUNK_SIZE) + iter_pos;

            let block_idx = chunk_bounds.index(iter_pos);
            let block_id = self.local_chunks[chunk_idx][block_idx];
            let block = &self.block_data[block_id as usize];

            for side in 0 .. 4 {
                let tile = block.tile(side);
                if tile == 0 {
                    continue;
                }

                let s = tile % ATLAS_SIZE;
                let t = tile / ATLAS_SIZE;
                emit_quad(buf, idx, Vertex {
                    corner: (0, 0),
                    pos: (pos.x as u8,
                          pos.y as u8,
                          pos.z as u8),
                    side: side as u8,
                    tex_coord: (s as u8,
                                t as u8),
                });
            }
        }

        // Stopped because the buffer is full.
        true
    }
}

impl<'a> Deref for GeomGen<'a> {
    type Target = GeomGenState;
    fn deref(&self) -> &GeomGenState {
        &self.state
    }
}

impl<'a> DerefMut for GeomGen<'a> {
    fn deref_mut(&mut self) -> &mut GeomGenState {
        &mut self.state
    }
}
