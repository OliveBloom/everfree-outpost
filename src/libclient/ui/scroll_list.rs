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

    fn calc_bounds_and_offset<D: ScrollListDyn>(&self,
                                                dyn: &D,
                                                height: i32) -> (usize, usize, i32) {
        // Calculate the visible index range as if the view was centered on the focus element.
        // Then adjust if it turns out the list is scrolled all the way to the top/bottom.

        let center_y = (height - ENTRY_HEIGHT) / 2;

        // Number of entries that can fit above the focus.
        let space_before = ((center_y + ENTRY_HEIGHT - 1) / ENTRY_HEIGHT) as usize;
        // Number of existing entries before the focus.
        let max_before = self.focus;

        let space_after = ((height - center_y + ENTRY_HEIGHT - 1) / ENTRY_HEIGHT) as usize;
        let max_after = dyn.len() - self.focus;

        // Number of visible entries when the list is scrolled all the way to one end.
        let end_count = ((height + ENTRY_HEIGHT - 1) / ENTRY_HEIGHT) as usize;

        if space_before > max_before {
            // List is scrolled all the way to the top.
            (0,
             cmp::min(dyn.len(), end_count),
             0)
        } else if space_after > max_after {
            // List is scrolled all the way to the bottom.
            let count = cmp::min(dyn.len(), end_count);
            (dyn.len() - count,
             dyn.len(),
             height - count as i32 * ENTRY_HEIGHT)
        } else {
            // List is centered on the focus.
            (self.focus - space_before,
             self.focus + space_after,
             center_y - space_before as i32 * ENTRY_HEIGHT)
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
        let height = bounds.size().y;

        let (start, end, base_offset) = self.state.calc_bounds_and_offset(self.dyn, height);
        assert!(0 <= start && start <= end && end < self.dyn.len(),
                "bad start/end: expected 0 <= {} <= {} < {}", start, end, self.dyn.len());

        for idx in start .. end {
            let x = bounds.min.x;
            let y = bounds.min.y + base_offset + (idx - start) as i32 * ENTRY_HEIGHT;
            let pos = V2::new(x, y);

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
