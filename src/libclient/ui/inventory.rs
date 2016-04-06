use std::prelude::v1::*;
use std::cmp;

use physics::v3::{V2, scalar, Region, Align};

use inventory::Item;
use ui::atlas;
use ui::geom::Geom;
use ui::input::{KeyAction, EventStatus};
use ui::item;
use ui::widget::*;


#[derive(Clone, Copy)]
struct Slot;

#[derive(Clone, Copy)]
enum SlotStatus {
    Inactive,
    Active,
    Semiactive,
}

#[derive(Clone, Copy)]
struct SlotDyn {
    item: Item,
    status: SlotStatus,
}

impl Slot {
    pub fn size() -> V2 {
        item::ItemDisplay::size() + scalar(2 * 2)
    }
}

impl<'a> Widget for WidgetPack<'a, Slot, SlotDyn> {
    fn size(&mut self) -> V2 { Slot::size() }

    fn walk_layout<V: Visitor>(&mut self, v: &mut V, pos: V2) {
        let dyn = item::ItemDyn::from_item(self.dyn.item);
        let mut child = WidgetPack::stateless(item::ItemDisplay, dyn);
        let rect = Region::sized(child.size()) + pos + scalar(2);
        v.visit(&mut child, rect);
    }

    fn render(&mut self, geom: &mut Geom, rect: Region<V2>) {
        let bg = match self.dyn.status {
            SlotStatus::Inactive => atlas::ITEM_SLOT_SQUARE_INACTIVE,
            SlotStatus::Active => atlas::ITEM_SLOT_SQUARE_ACTIVE,
            SlotStatus::Semiactive => atlas::ITEM_SLOT_SQUARE_SEMIACTIVE,
        };

        geom.draw_ui(bg, rect.min);
    }
}


#[derive(Clone, Copy)]
pub struct Grid {
    pub focus: usize,
    // Child `Slot`s are stateless
}

impl Grid {
    pub fn new() -> Grid {
        Grid {
            focus: 0,
        }
    }

    pub fn move_focus(&mut self, dir: V2, grid_size: V2, inv_len: usize) {
        let bounds = Region::sized(grid_size);
        let limit = cmp::min(inv_len, bounds.volume() as usize);
        let pos = bounds.from_index(cmp::min(self.focus, limit - 1));
        let new_pos = bounds.clamp_point(pos + dir);
        let new_focus = bounds.index(new_pos);
        self.focus = cmp::min(new_focus, limit - 1);
    }
}

pub trait GridDyn: Copy {
    fn grid_size(self) -> V2;
    fn len(self) -> usize;
    fn item(self, i: usize) -> Item;
}

impl<'a, D: GridDyn> Widget for WidgetPack<'a, Grid, D> {
    fn size(&mut self) -> V2 {
        Slot::size() * self.dyn.grid_size()
    }

    fn walk_layout<V: Visitor>(&mut self, v: &mut V, pos: V2) {
        let grid_bounds = Region::sized(self.dyn.grid_size());
        for (idx, slot_pos) in (0 .. self.dyn.len()).zip(grid_bounds.points()) {
            let dyn = SlotDyn {
                item: self.dyn.item(idx),
                status:
                    if idx == self.state.focus { SlotStatus::Active }
                    else { SlotStatus::Inactive },
            };

            let mut child = WidgetPack::stateless(Slot, dyn);
            let rect = Region::sized(child.size()) + pos + slot_pos * Slot::size();
            v.visit(&mut child, rect);
        }
    }

    fn render(&mut self, _geom: &mut Geom, _rect: Region<V2>) {
    }

    fn on_key(&mut self, key: KeyAction) -> EventStatus {
        use ui::input::KeyAction::*;
        let dir =
            match key {
                MoveUp(amt) =>      Some(V2::new(0, -(amt as i32))),
                MoveDown(amt) =>    Some(V2::new(0,  (amt as i32))),
                MoveLeft(amt) =>    Some(V2::new(-(amt as i32), 0)),
                MoveRight(amt) =>   Some(V2::new( (amt as i32), 0)),
                _ =>                None,
            };

        if let Some(dir) = dir {
            self.state.move_focus(dir, self.dyn.grid_size(), self.dyn.len());
            return EventStatus::Handled;
        }

        EventStatus::Unhandled
    }
}
