use std::prelude::v1::*;

use physics::v3::{V2, scalar, Region, Align};

use client::ClientObj;
use input::EventStatus;
use ui::{Context, DragData};
use ui::atlas;
use ui::geom::Geom;
use ui::item;
use ui::widget::*;


#[derive(Clone, Copy)]
struct Slot;

#[derive(Clone, Copy)]
struct SlotDyn {
    item_dyn: item::ItemDyn,
    color: u8,
}

impl Slot {
    pub fn size() -> V2 {
        item::ItemDisplay::size() + scalar(4 * 2)
    }
}

impl<'a> Widget for WidgetPack<'a, Slot, SlotDyn> {
    fn size(&mut self) -> V2 { Slot::size() }

    fn walk_layout<V: Visitor>(&mut self, v: &mut V, pos: V2) {
        let mut child = WidgetPack::stateless(item::ItemDisplay, &self.dyn.item_dyn);
        let rect = Region::sized(child.size()) + pos + scalar(4);
        v.visit(&mut child, rect);
    }

    fn render(&mut self, geom: &mut Geom, rect: Region<V2>) {
        let bg = match self.dyn.color {
            0 => atlas::HOTBAR_BOX_YELLOW,
            1 => atlas::HOTBAR_BOX_GREEN,
            2 => atlas::HOTBAR_BOX_BLUE,
            3 => atlas::HOTBAR_BOX_RED,
            _ => unreachable!(),
        };

        geom.draw_ui(bg, rect.min);
    }
}


pub struct Hotbar;

pub struct SlotInfo {
    pub item_id: u16,
    pub quantity: Option<u16>,
    pub is_active_item: bool,
    pub is_active_ability: bool,
}

pub trait HotbarDyn {
    fn slot_info(&self, idx: u8) -> SlotInfo;
}

impl Hotbar {
    pub fn size() -> V2 {
        // Width of all slots, including the gaps in between and at start and end.
        let w = 9 * Slot::size().y + 10 * 1;
        let h = Slot::size().y;
        V2::new(w, h)
    }
}

impl<'a, D: HotbarDyn> Widget for WidgetPack<'a, Hotbar, D> {
    fn size(&mut self) -> V2 { Hotbar::size() }

    fn walk_layout<V: Visitor>(&mut self, v: &mut V, pos: V2) {
        let base = pos + V2::new(1, 0);
        let child_size = Slot::size();
        let step = V2::new(child_size.x + 1, 0);

        for i in 0 .. 9 {
            let info = self.dyn.slot_info(i);
            let dyn = SlotDyn {
                item_dyn: item::ItemDyn::new(info.item_id, info.quantity),
                color:
                    if info.is_active_item { 1 }
                    else if info.is_active_ability { 2 }
                    else { 0 },
            };

            let mut child = WidgetPack::stateless(Slot, &dyn);
            let rect = Region::sized(child_size) + base + step * scalar(i as i32);
            v.visit(&mut child, rect);
        }
    }

    fn render(&mut self, geom: &mut Geom, rect: Region<V2>) {
        let bar_h = atlas::HOTBAR_BAR.size.1;
        let bar_rect = Region::sized(V2::new(rect.size().x, bar_h as i32));

        let bar_dest = bar_rect.align(rect, Align::Center, Align::Center);
        geom.draw_ui_tiled(atlas::HOTBAR_BAR, bar_dest);
    }

    fn on_drop(&mut self, ctx: &mut Context, rect: Region<V2>, data: &DragData) -> EventStatus {
        let DragData { src_inv, src_slot } = *data;

        let x_off = ctx.mouse_pos.x - rect.min.x;
        let slot_idx = (x_off - 1) / (Slot::size().x + 1);
        if slot_idx < 0 || slot_idx >= 9 {
            return EventStatus::Unhandled;
        }
        let slot_idx = slot_idx as u8;

        EventStatus::Action(box move |c: &mut ClientObj| {
            c.handle_hotbar_drop(src_inv, src_slot, slot_idx);
        })
    }

    fn check_drop(&mut self, ctx: &Context, rect: Region<V2>, _data: &DragData) -> bool {
        let x_off = ctx.mouse_pos.x - rect.min.x;
        let slot_idx = (x_off - 1) / (Slot::size().x + 1);
        if slot_idx < 0 || slot_idx >= 9 {
            return false
        }
        true
    }
}
