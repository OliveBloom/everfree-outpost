use std::prelude::v1::*;

use physics::v3::{V2, scalar, Region, Align};

use client::ClientObj;
use data::Data;
use platform::{Config, ConfigKey};
use ui::{Context, DragData};
use ui::atlas;
use ui::geom::Geom;
use ui::input::EventStatus;
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
        let mut child = WidgetPack::stateless(item::ItemDisplay, self.dyn.item_dyn);
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

pub trait HotbarDyn: Copy {
    fn slot_info(self, idx: u8) -> SlotInfo;
}

impl Hotbar {
    pub fn size() -> V2 {
        let w = Slot::size().x;

        // Height of all slots, including the gaps in between.
        let slot_h = 9 * Slot::size().y + 8 * 1;
        let h = 8 + slot_h + 8;

        V2::new(w, h)
    }
}

impl<'a, D: HotbarDyn> Widget for WidgetPack<'a, Hotbar, D> {
    fn size(&mut self) -> V2 { Hotbar::size() }

    fn walk_layout<V: Visitor>(&mut self, v: &mut V, pos: V2) {
        let base = pos + V2::new(0, 8);
        let child_size = Slot::size();
        let step = V2::new(0, child_size.y + 1);

        for i in 0 .. 9 {
            let info = self.dyn.slot_info(i);
            let dyn = SlotDyn {
                item_dyn: item::ItemDyn::new(info.item_id, info.quantity),
                color:
                    if info.is_active_item { 1 }
                    else if info.is_active_ability { 2 }
                    else { 0 },
            };

            let mut child = WidgetPack::stateless(Slot, dyn);
            let rect = Region::sized(child_size) + base + step * scalar(i as i32);
            v.visit(&mut child, rect);
        }
    }

    fn render(&mut self, geom: &mut Geom, rect: Region<V2>) {
        let (cap_w, cap_h) = atlas::HOTBAR_CAP_TOP.size;
        let cap_rect = Region::sized(V2::new(cap_w as i32, cap_h as i32));

        geom.draw_ui(atlas::HOTBAR_CAP_TOP,
                     cap_rect.align(rect, Align::Center, Align::Start).min);
        geom.draw_ui(atlas::HOTBAR_CAP_BOTTOM,
                     cap_rect.align(rect, Align::Center, Align::End).min);

        let bar_w = atlas::HOTBAR_BAR.size.0;
        let bar_rect = Region::sized(V2::new(bar_w as i32, rect.size().y - 2 * 7));

        let bar_dest = bar_rect.align(rect, Align::Center, Align::Center);
        geom.draw_ui_tiled(atlas::HOTBAR_BAR, bar_dest);
    }

    fn on_drop(&mut self, ctx: &mut Context, rect: Region<V2>, data: &DragData) -> EventStatus {
        let DragData { src_inv, src_slot } = *data;

        let y_off = ctx.mouse_pos.y - rect.min.y;
        let slot_idx = (y_off - 8) / (Slot::size().y + 1);
        if slot_idx < 0 || slot_idx >= 9 {
            return EventStatus::Unhandled;
        }
        let slot_idx = slot_idx as u8;

        EventStatus::Action(box move |c: &mut ClientObj| {
            c.handle_hotbar_drop(src_inv, src_slot, slot_idx);
        })
    }

    fn check_drop(&mut self, ctx: &Context, rect: Region<V2>, data: &DragData) -> bool {
        let y_off = ctx.mouse_pos.y - rect.min.y;
        let slot_idx = (y_off - 8) / (Slot::size().y + 1);
        if slot_idx < 0 || slot_idx >= 9 {
            return false;
        }
        true
    }
}
