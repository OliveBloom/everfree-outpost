use std::prelude::v1::*;
use std::mem;

use physics::v3::{V2, scalar, Region};

use fonts::{FontMetrics, FontMetricsExt};

use super::atlas::AtlasEntry;


pub struct Vertex {
    src_pos: (u16, u16),
    src_size: (u8, u8),
    sheet: u8,
    _pad1: u8,

    dest: (i16, i16),
    offset: (u16, u16),
}

pub enum Special {
    DebugFrameGraph {
        rect: Region<V2>,
        cur: u8,
        last: u8,
        last_time: u16,
        last_interval: u16,
    },
}

pub struct Geom {
    geom: Vec<Vertex>,
    special: Vec<Special>,
}


/// Size in pixels of each item.
const ITEM_SIZE: u16 = 16;
/// Number of item images per row/column of the sheet.
const ITEM_SHEET_SIZE: u16 = 32;

const ITEM_SHEET: u8 = 0;
const UI_SHEET: u8 = 1;
const FONT_SHEET: u8 = 2;

impl Geom {
    pub fn new() -> Geom {
        Geom {
            geom: Vec::new(),
            special: Vec::new(),
        }
    }

    fn emit_quad(&mut self, entry: AtlasEntry, sheet: u8, dest_pos: V2, dest_size: V2) {
        let dx = dest_pos.x as i16;
        let dy = dest_pos.y as i16;
        let dw = dest_size.x as u16;
        let dh = dest_size.y as u16;

        let mut go = |ox, oy| {
            self.geom.push(Vertex {
                src_pos: entry.pos,
                src_size: entry.size,
                sheet: sheet,
                _pad1: 0,
                dest: (dx, dy),
                offset: (ox, oy),
            });
        };

        go( 0,  0);
        go( 0, dh);
        go(dw,  0);

        go(dw,  0);
        go( 0, dh);
        go(dw, dh);
    }

    pub fn draw_ui(&mut self, entry: AtlasEntry, pos: V2) {
        let size = V2::new(entry.size.0 as i32,
                           entry.size.1 as i32);
        self.emit_quad(entry, UI_SHEET, pos, size);
    }

    pub fn draw_ui_tiled(&mut self, entry: AtlasEntry, dest: Region<V2>) {
        self.emit_quad(entry, UI_SHEET, dest.min, dest.size());
    }

    pub fn draw_item(&mut self, item_id: u16, pos: V2) {
        let x = item_id % ITEM_SHEET_SIZE;
        let y = item_id / ITEM_SHEET_SIZE;

        let entry = AtlasEntry {
            pos: (x * ITEM_SIZE, y * ITEM_SIZE),
            size: (ITEM_SIZE as u8, ITEM_SIZE as u8),
        };
        self.emit_quad(entry, ITEM_SHEET, pos, scalar(ITEM_SIZE as i32));
    }

    pub fn draw_char(&mut self, font: &FontMetrics, idx: usize, pos: V2) {
        let x = font.xs[idx];
        let y = font.y as u16;
        let width = font.widths[idx];
        let height = font.height;

        let entry = AtlasEntry {
            pos: (x, y),
            size: (width, height),
        };

        self.emit_quad(entry, FONT_SHEET, pos, V2::new(width as i32, height as i32));
    }

    pub fn draw_str(&mut self, font: &FontMetrics, s: &str, pos: V2) {
        for (idx, offset) in font.iter_str(s) {
            if let Some(idx) = idx {
                self.draw_char(font, idx, pos + V2::new(offset as i32, 0));
            }
        }
    }

    pub fn special(&mut self, special: Special) {
        self.special.push(special)
    }

    pub fn unwrap(self) -> (Vec<Vertex>, Vec<Special>) {
        (self.geom, self.special)
    }
}
