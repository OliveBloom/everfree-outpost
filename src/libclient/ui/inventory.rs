use std::prelude::v1::*;

use physics::v3::{V2, scalar, Region, Align};

use ui::atlas;
use ui::geom::Geom;
use ui::item::{ItemDisplay, ItemDyn};
use ui::widget::*;


#[derive(Clone, Copy)]
pub struct Slot;

#[derive(Clone, Copy)]
pub enum SlotState {
    Inactive,
    Active,
    Semiactive,
}

pub trait SlotDyn: Copy {
    type ItemDyn: ItemDyn;
    fn item(self) -> Self::ItemDyn;
    fn state(self) -> SlotState;
}

impl Slot {
    pub fn size() -> V2 {
        ItemDisplay::size() + scalar(2 * 2)
    }
}

impl<D: SlotDyn> Widget for WidgetPack<Slot, D> {
    fn size(self) -> V2 { Slot::size() }

    fn walk_layout<V: Visitor>(self, v: &mut V, pos: V2) {
        let child = WidgetPack::new(ItemDisplay, self.dyn.item());
        let child_size = child.size();
        let child_pos = pos + scalar(2);
        v.visit(child, Region::new(child_pos, child_pos + child_size));
    }

    fn render(self, geom: &mut Geom, rect: Region<V2>) {
        let bg = match self.dyn.state() {
            SlotState::Inactive => atlas::ITEM_SLOT_SQUARE_INACTIVE,
            SlotState::Active => atlas::ITEM_SLOT_SQUARE_ACTIVE,
            SlotState::Semiactive => atlas::ITEM_SLOT_SQUARE_SEMIACTIVE,
        };

        geom.draw_ui(bg, rect.min);
    }
}


#[derive(Clone, Copy)]
pub struct Grid(pub u8);

pub trait GridDyn: Copy {
    type SlotDyn: SlotDyn;
    fn len(self) -> usize;
    fn slot(self, i: usize) -> Self::SlotDyn;
}

impl Grid {
    fn grid_size(self, len: usize) -> V2 {
        let Grid(cols) = self;
        let cols = cols as usize;
        let rows = (len + cols - 1) / cols;
        V2::new(cols as i32, rows as i32)
    }
}

impl<D: GridDyn> Widget for WidgetPack<Grid, D> {
    fn size(self) -> V2 {
        Slot::size() * self.w.grid_size(self.dyn.len())
    }

    fn walk_layout<V: Visitor>(self, v: &mut V, pos: V2) {
        let grid_bounds = Region::sized(self.w.grid_size(self.dyn.len()));
        for idx in 0 .. self.dyn.len() {
            let slot_pos = grid_bounds.from_index(idx);
            let child = WidgetPack::new(Slot, self.dyn.slot(idx));
            let child_size = child.size();
            let child_pos = pos + slot_pos * Slot::size();
            v.visit(child, Region::new(child_pos, child_pos + child_size));
        }
    }

    fn render(self, _geom: &mut Geom, _rect: Region<V2>) {
    }
}
