use std::prelude::v1::*;
use std::cmp;

use physics::v3::{V2, Vn, scalar, Region};

use client::ClientObj;
use inventory::Item;
use ui::{Context, DragData};
use ui::atlas;
use ui::geom::Geom;
use ui::input::{KeyEvent, ButtonEvent, EventStatus};
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
        let mut child = WidgetPack::stateless(item::ItemDisplay, &dyn);
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

pub trait GridDyn {
    fn grid_size(&self) -> V2;
    fn len(&self) -> usize;
    fn item(&self, i: usize) -> Item;
    fn active(&self) -> bool;
    fn inv_id(&self) -> Option<u32>;
}

impl<'a, D: GridDyn> WidgetPack<'a, Grid, D> {
    fn slot_at_offset(&self, offset: V2) -> Option<usize> {
        let slot_pos = offset.div_floor(Slot::size());

        let grid_bounds = Region::sized(self.dyn.grid_size());
        if !grid_bounds.contains(slot_pos) {
            return None;
        }

        let idx = grid_bounds.index(slot_pos);
        if idx >= self.dyn.len() {
            return None;
        }

        Some(idx)
    }

    fn maybe_start_drag(&self, ctx: &mut Context, idx: usize) {
        if let Some(src_id) = self.dyn.inv_id() {
            let item = self.dyn.item(idx);
            if item.id != 0 {
                ctx.drag_item(src_id, idx);
            }
        }
    }
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
                    if idx == self.state.focus {
                        if self.dyn.active() { SlotStatus::Active }
                        else { SlotStatus::Semiactive }
                    }
                    else { SlotStatus::Inactive },
            };

            let mut child = WidgetPack::stateless(Slot, &dyn);
            let rect = Region::sized(child.size()) + pos + slot_pos * Slot::size();
            v.visit(&mut child, rect);
        }
    }

    fn render(&mut self, _geom: &mut Geom, _rect: Region<V2>) {
    }

    fn on_key(&mut self, key: KeyEvent) -> EventStatus {
        use ui::input::KeyAction::*;
        let amt = if key.shift() { 10 } else { 1 };
        let dir =
            match key.code {
                MoveUp =>       Some(V2::new(0, -amt)),
                MoveDown =>     Some(V2::new(0,  amt)),
                MoveLeft =>     Some(V2::new(-amt, 0)),
                MoveRight =>    Some(V2::new( amt, 0)),
                _ =>            None,
            };

        if let Some(dir) = dir {
            let old_focus = self.state.focus;
            self.state.move_focus(dir, self.dyn.grid_size(), self.dyn.len());
            if self.state.focus != old_focus {
                return EventStatus::Handled;
            }
        }

        EventStatus::Unhandled
    }

    fn on_mouse_move(&mut self, ctx: &mut Context, rect: Region<V2>) -> EventStatus {
        let idx = match self.slot_at_offset(ctx.mouse_pos - rect.min) {
            Some(x) => x,
            None => return EventStatus::Unhandled,
        };

        if ctx.moved_while_down() && !ctx.dragging() {
            self.maybe_start_drag(ctx, idx);
        }

        self.state.focus = idx;
        EventStatus::Handled
    }

    fn on_mouse_up(&mut self,
                   ctx: &mut Context,
                   rect: Region<V2>,
                   _evt: ButtonEvent) -> EventStatus {
        let idx = match self.slot_at_offset(ctx.mouse_pos - rect.min) {
            Some(x) => x,
            None => return EventStatus::Unhandled,
        };

        if !ctx.moved_while_down() && !ctx.dragging() {
            // The user clicked without dragging.  Pick up the item.
            self.maybe_start_drag(ctx, idx);
        }

        EventStatus::Handled
    }

    fn on_drop(&mut self, ctx: &mut Context, rect: Region<V2>, data: &DragData) -> EventStatus {
        let DragData { src_inv, src_slot } = *data;
        let dest_inv = match self.dyn.inv_id() {
            Some(x) => x,
            None => return EventStatus::Unhandled,
        };
        let dest_slot = match self.slot_at_offset(ctx.mouse_pos - rect.min) {
            Some(x) => x,
            None => return EventStatus::Unhandled,
        };
        // Bit of a hack - just move as much as possible.
        // TODO: change the MoveItem API to take an enum, All / Half / One, and compute the exact
        // quantity server-side
        let amount = 255;

        EventStatus::Action(box move |c: &mut ClientObj| {
            c.platform().send_move_item(src_inv, src_slot, dest_inv, dest_slot, amount);
        })
    }

    fn check_drop(&mut self, ctx: &Context, rect: Region<V2>, _data: &DragData) -> bool {
        // Just check that the mouse is over an actual slot.
        self.slot_at_offset(ctx.mouse_pos - rect.min).is_some()
    }
}
