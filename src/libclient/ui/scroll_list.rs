use std::prelude::v1::*;
use std::cmp;

use physics::v3::{V2, Vn, scalar, Region};

use client::ClientObj;
use fonts;
use inventory::Item;
use ui::{Context, DragData};
use ui::atlas;
use ui::geom::Geom;
use ui::input::{KeyEvent, EventStatus};
use ui::item;
use ui::widget::*;


#[derive(Clone, Copy)]
struct Entry;

struct EntryDyn<'a> {
    width: i32,
    label: &'a str,
}

const ENTRY_HEIGHT: i32 = 13;

impl<'a> Widget for WidgetPack<'a, Entry, EntryDyn<'a>> {
    fn size(&mut self) -> V2 {
        V2::new(self.dyn.width, ENTRY_HEIGHT)
    }

    fn walk_layout<V: Visitor>(&mut self, v: &mut V, pos: V2) {
    }

    fn render(&mut self, geom: &mut Geom, rect: Region<V2>) {
        geom.draw_ui(atlas::SCROLL_LIST_ENTRY_LEFT, rect.min);
        geom.draw_ui(atlas::SCROLL_LIST_ENTRY_RIGHT,
                     V2::new(rect.max.x - 1, rect.min.y));
        geom.draw_ui_tiled(atlas::SCROLL_LIST_ENTRY_MID,
                           Region::new(rect.min + V2::new(2, 0),
                                       rect.max - V2::new(1, 0)));

        let offset = V2::new(2, 2);
        geom.draw_str(&fonts::NAME, self.dyn.label, rect.min + offset);
    }
}


#[derive(Clone)]
pub struct ScrollList {
    pub focus: usize,
    pub size: V2,
    // Child items are stateless
}

impl ScrollList {
    pub fn new(size: V2) -> ScrollList {
        ScrollList {
            focus: 0,
            size: size,
        }
    }
}

pub trait ScrollListDyn {
    fn get(&self, idx: usize) -> &str;
    fn len(&self) -> usize;
}

impl<'a, D: ScrollListDyn> Widget for WidgetPack<'a, ScrollList, D> {
    fn size(&mut self) -> V2 {
        self.state.size
    }

    fn walk_layout<V: Visitor>(&mut self, v: &mut V, pos: V2) {
        let outer_bounds = Region::sized(self.state.size) + pos;
        let bounds = outer_bounds.inset(3, 3, 3, 3);
        let width = bounds.size().x;

        // Put the focused entry at the center of the view
        let focus_x = bounds.min.x;
        let focus_y = (bounds.min.y + bounds.max.y - ENTRY_HEIGHT) / 2;
        let focus = self.state.focus as isize;

        // Compute the number of visible entries
        let num_before = (focus_y - bounds.min.y + ENTRY_HEIGHT - 1) / ENTRY_HEIGHT;
        let num_after = (bounds.max.y - focus_y + ENTRY_HEIGHT - 1) / ENTRY_HEIGHT;
        let start = focus - num_before as isize;
        let end = focus + num_after as isize;

        for idx in start .. end {
            if idx < 0 || idx >= self.dyn.len() as isize {
                continue;
            }

            let offset = idx - focus;
            let y = focus_y + offset as i32 * ENTRY_HEIGHT;
            let pos = V2::new(focus_x, y);

            let dyn = EntryDyn {
                width: width,
                label: self.dyn.get(idx as usize),
            };
            let mut child = WidgetPack::stateless(Entry, &dyn);
            let rect = Region::sized(child.size()) + pos;
            v.visit_clipped(&mut child, rect, bounds);
        }
    }

    fn render(&mut self, geom: &mut Geom, rect: Region<V2>) {
        geom.draw_ui(atlas::SCROLL_LIST_BORDER_NW, rect.inset(0, -3, 0, -3).min);
        geom.draw_ui(atlas::SCROLL_LIST_BORDER_NE, rect.inset(-3, 0, 0, -3).min);
        geom.draw_ui(atlas::SCROLL_LIST_BORDER_SW, rect.inset(0, -3, -3, 0).min);
        geom.draw_ui(atlas::SCROLL_LIST_BORDER_SE, rect.inset(-3, 0, -3, 0).min);

        geom.draw_ui_tiled(atlas::SCROLL_LIST_BORDER_N, rect.inset(3, 3, 0, -3));
        geom.draw_ui_tiled(atlas::SCROLL_LIST_BORDER_S, rect.inset(3, 3, -3, 0));
        geom.draw_ui_tiled(atlas::SCROLL_LIST_BORDER_W, rect.inset(0, -3, 3, 3));
        geom.draw_ui_tiled(atlas::SCROLL_LIST_BORDER_E, rect.inset(-3, 0, 3, 3));
    }
}
