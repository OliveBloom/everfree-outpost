use std::prelude::v1::*;

use physics::v3::{V2, scalar, Region, Align};

use ui::atlas;
use ui::geom::Geom;
use ui::item;
use ui::widget::*;


#[derive(Clone, Copy)]
pub struct Slot;

#[derive(Clone, Copy)]
pub enum SlotStatus {
    Inactive,
    Active,
    Semiactive,
}

pub trait SlotDyn: Copy {
    type ItemDyn: item::ItemDyn;
    fn item(self) -> Self::ItemDyn;
    fn status(self) -> SlotStatus;
}

impl Slot {
    pub fn size() -> V2 {
        item::ItemDisplay::size() + scalar(2 * 2)
    }
}

impl<'a, D: SlotDyn> Widget for WidgetPack<'a, Slot, D> {
    fn size(&mut self) -> V2 { Slot::size() }

    fn walk_layout<V: Visitor>(&mut self, v: &mut V, pos: V2) {
        let mut child = WidgetPack::stateless(item::ItemDisplay, self.dyn.item());
        let child_rect = Region::sized(child.size()) + pos + scalar(2);
        v.visit(&mut child, child_rect);
    }

    fn render(&mut self, geom: &mut Geom, rect: Region<V2>) {
        let bg = match self.dyn.status() {
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
}

pub trait GridDyn: Copy {
    type ItemDyn: item::ItemDyn;
    fn grid_size(self) -> V2;
    fn len(self) -> usize;
    fn item(self, i: usize) -> Self::ItemDyn;
}

impl<'a, D: GridDyn> Widget for WidgetPack<'a, Grid, D> {
    fn size(&mut self) -> V2 {
        Slot::size() * self.dyn.grid_size()
    }

    fn walk_layout<V: Visitor>(&mut self, v: &mut V, pos: V2) {
        let grid_bounds = Region::sized(self.dyn.grid_size());
        for (idx, slot_pos) in (0 .. self.dyn.len()).zip(grid_bounds.points()) {
            let status =
                if idx == self.state.focus { SlotStatus::Active }
                else { SlotStatus::Inactive };
            let child_dyn = GridSlot::new(self.dyn.item(idx), status);
            let mut child = WidgetPack::stateless(Slot, child_dyn);
            let child_rect = Region::sized(child.size()) + pos + slot_pos * Slot::size();
            v.visit(&mut child, child_rect);
        }
    }

    fn render(&mut self, _geom: &mut Geom, _rect: Region<V2>) {
    }
}


#[derive(Clone, Copy)]
struct GridSlot<D: item::ItemDyn> {
    item: D,
    status: SlotStatus,
}

impl<D: item::ItemDyn> SlotDyn for GridSlot<D> {
    type ItemDyn = D;
    fn item(self) -> D { self.item }
    fn status(self) -> SlotStatus { self.status }
}

impl<D: item::ItemDyn> GridSlot<D> {
    fn new(item: D, status: SlotStatus) -> GridSlot<D> {
        GridSlot {
            item: item,
            status: status,
        }
    }
}
