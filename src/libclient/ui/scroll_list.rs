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
    active: bool,
    enabled: bool,
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
        if self.dyn.active {
            geom.draw_ui(atlas::SCROLL_LIST_MARKER,
                         rect.min + V2::new(2, 2));
        }

        let font =
            if self.dyn.enabled { &fonts::NAME }
            else { &fonts::DEFAULT_GRAY };
        let offset = V2::new(9, 2);
        geom.draw_str(font, self.dyn.label, rect.min + offset);
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
    fn is_enabled(&self, idx: usize) -> bool { true }
    fn len(&self) -> usize;
}

impl<'a, D: ScrollListDyn> Widget for WidgetPack<'a, ScrollList, D> {
    fn size(&mut self) -> V2 {
        self.state.size
    }

    fn walk_layout<V: Visitor>(&mut self, v: &mut V, pos: V2) {
        let bounds = Region::sized(self.state.size) + pos;
        let body_bounds = bounds.inset(3, 3 + 11, 3, 3);
        let scroll_bounds = bounds.inset(-10, 0, 1, 1);

        let width = body_bounds.size().x;
        let height = body_bounds.size().y;

        let (start, end, base_offset) = self.state.calc_bounds_and_offset(self.dyn, height);
        assert!(0 <= start && start <= end && end <= self.dyn.len(),
                "bad start/end: expected 0 <= {} <= {} <= {}", start, end, self.dyn.len());

        for idx in start .. end {
            let y_off = base_offset + (idx - start) as i32 * ENTRY_HEIGHT;
            let pos = body_bounds.min + V2::new(0, y_off);

            let dyn = EntryDyn {
                width: width,
                label: self.dyn.get(idx as usize),
                active: idx == self.state.focus,
                enabled: self.dyn.is_enabled(idx as usize),
            };
            let mut child = WidgetPack::stateless(Entry, &dyn);
            let rect = Region::sized(child.size()) + pos;
            v.visit_clipped(&mut child, rect, body_bounds);
        }

        {
            let mut state = ScrollBar::new(0,
                                           self.dyn.len(),
                                           &mut self.state.focus,
                                           scroll_bounds.size());
            let dyn = ();
            let mut child = WidgetPack::new(&mut state, &dyn);
            v.visit(&mut child, scroll_bounds);
        }
    }

    fn render(&mut self, geom: &mut Geom, rect: Region<V2>) {
        let body = rect.inset(0, 11, 0, 0);
        let scroll = rect.inset(-10, 0, 1, 1);

        geom.draw_ui(atlas::SCROLL_LIST_BORDER_NW, body.inset(0, -3, 0, -3).min);
        geom.draw_ui(atlas::SCROLL_LIST_BORDER_NE, body.inset(-3, 0, 0, -3).min);
        geom.draw_ui(atlas::SCROLL_LIST_BORDER_SW, body.inset(0, -3, -3, 0).min);
        geom.draw_ui(atlas::SCROLL_LIST_BORDER_SE, body.inset(-3, 0, -3, 0).min);

        geom.draw_ui_tiled(atlas::SCROLL_LIST_BORDER_N, body.inset(3, 3, 0, -3));
        geom.draw_ui_tiled(atlas::SCROLL_LIST_BORDER_S, body.inset(3, 3, -3, 0));
        geom.draw_ui_tiled(atlas::SCROLL_LIST_BORDER_W, body.inset(0, -3, 3, 3));
        geom.draw_ui_tiled(atlas::SCROLL_LIST_BORDER_E, body.inset(-3, 0, 3, 3));

        // TODO: fill background
    }

    fn on_key(&mut self, key: KeyEvent) -> EventStatus {
        use ui::input::KeyAction::*;

        let amt = if key.shift() { 10 } else { 1 };
        let old_focus = self.state.focus;
        match key.code {
            MoveUp => {
                self.state.focus -= cmp::min(amt, self.state.focus);
            },

            MoveDown => {
                self.state.focus = cmp::min(self.state.focus + amt, self.dyn.len() - 1);
            },

            _ => {},
        }


        if self.state.focus != old_focus {
            EventStatus::Handled
        } else {
            EventStatus::Unhandled
        }
    }
}


struct ScrollBar<'a> {
    min: usize,
    max: usize,
    cur: &'a mut usize,
    size: V2,
}

const SCROLL_BAR_CAP_SIZE: i32 = atlas::SCROLL_BAR_CAP.size.1 as i32;

impl<'a> ScrollBar<'a> {
    pub fn new(min: usize, max: usize, cur: &'a mut usize, size: V2) -> ScrollBar<'a> {
        assert!(min <= max);
        ScrollBar {
            min: min,
            max: max,
            cur: cur,
            size: size,
        }
    }

    fn thumb_pos(&self, height: i32) -> i32 {
        if self.min == self.max {
            return 0;
        }
        ((*self.cur - self.min) * height as usize / (self.max - 1 - self.min)) as i32
    }
}

impl<'a, 'b> Widget for WidgetPack<'a, ScrollBar<'b>, ()> {
    fn size(&mut self) -> V2 {
        self.state.size
    }

    fn walk_layout<V: Visitor>(&mut self, v: &mut V, pos: V2) {
    }

    fn render(&mut self, geom: &mut Geom, rect: Region<V2>) {
        const CAP_SIZE: i32 = SCROLL_BAR_CAP_SIZE;
        let bar = rect.inset(3, 3, CAP_SIZE, CAP_SIZE);

        const THUMB_INNER_HEIGHT: i32 = atlas::SCROLL_BAR_THUMB.size.1 as i32 - 2;
        let thumb_offset = self.state.thumb_pos(bar.size().y - THUMB_INNER_HEIGHT);
        let thumb_y = bar.min.y - 1 + thumb_offset;

        geom.draw_ui_tiled(atlas::SCROLL_BAR_BAR_BELOW, bar);

        geom.draw_ui(atlas::SCROLL_BAR_CAP, V2::new(rect.min.x + 1, rect.min.y));
        geom.draw_ui(atlas::SCROLL_BAR_CAP, V2::new(rect.min.x + 1, rect.max.y - CAP_SIZE));

        geom.draw_ui(atlas::SCROLL_BAR_THUMB, V2::new(rect.min.x, thumb_y));
    }
}
