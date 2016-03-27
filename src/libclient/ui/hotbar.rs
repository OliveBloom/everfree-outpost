use std::prelude::v1::*;

use physics::v3::{V2, scalar, Region, Align};

use ui::atlas;
use ui::geom::Geom;
use ui::item;
use ui::widget::*;


#[derive(Clone, Copy)]
pub struct Slot;

pub trait SlotDyn: Copy {
    type ItemDyn: item::ItemDyn;
    fn item(self) -> Self::ItemDyn;
    fn color(self) -> u8;
}

impl Slot {
    pub fn size() -> V2 {
        item::ItemDisplay::size() + scalar(4 * 2)
    }
}

impl<'a, D: SlotDyn> Widget for WidgetPack<'a, Slot, D> {
    fn size(&mut self) -> V2 { Slot::size() }

    fn walk_layout<V: Visitor>(&mut self, v: &mut V, pos: V2) {
        let mut child = WidgetPack::stateless(item::ItemDisplay, self.dyn.item());
        let child_rect = Region::sized(child.size()) + pos + scalar(4);
        v.visit(&mut child, child_rect);
    }

    fn render(&mut self, geom: &mut Geom, rect: Region<V2>) {
        let bg = match self.dyn.color() {
            0 => atlas::HOTBAR_BOX_YELLOW,
            1 => atlas::HOTBAR_BOX_GREEN,
            2 => atlas::HOTBAR_BOX_BLUE,
            3 => atlas::HOTBAR_BOX_RED,
            _ => unreachable!(),
        };

        geom.draw_ui(bg, rect.min);
    }
}


#[derive(Clone, Copy)]
pub struct HotbarSlot {
    pub item_id: u16,
    pub is_ability: bool,
}

pub struct Hotbar {
    pub slots: [HotbarSlot; 9],
    pub cur_item: i8,
    pub cur_ability: i8,
}

impl Hotbar {
    pub fn new() -> Hotbar {
        Hotbar {
            slots: [HotbarSlot { item_id: 0, is_ability: false }; 9],
            cur_item: -1,
            cur_ability: -1,
        }
    }
}

pub trait HotbarDyn: Copy {
    fn item_count(self, item_id: u16, ability: bool) -> u16;
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
            let slot = self.state.slots[i];
            let qty = self.dyn.item_count(slot.item_id, slot.is_ability);
            let color =
                if i as i8 == self.state.cur_item { 1 }
                else if i as i8 == self.state.cur_ability { 1 }
                else { 0 };
            let child_dyn = SlotDynImpl::new(slot, qty, color);

            let mut child = WidgetPack::stateless(Slot, child_dyn);
            let child_rect = Region::sized(child_size) + base + step * scalar(i as i32);
            v.visit(&mut child, child_rect);
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
}


#[derive(Clone, Copy)]
struct SlotDynImpl {
    slot: HotbarSlot,
    quantity: u16,
    color: u8,
}

impl SlotDynImpl {
    fn new(slot: HotbarSlot, quantity: u16, color: u8) -> SlotDynImpl {
        SlotDynImpl {
            slot: slot,
            quantity: quantity,
            color: color,
        }
    }
}

impl item::ItemDyn for SlotDynImpl {
    fn item_id(self) -> u16 { self.slot.item_id }

    fn quantity(self) -> Option<u16> {
        if self.slot.is_ability || self.slot.item_id == 0 {
            None
        } else {
            Some(self.quantity)
        }
    }
}

impl SlotDyn for SlotDynImpl {
    type ItemDyn = Self;
    fn item(self) -> Self { self }
    fn color(self) -> u8 { self.color }
}
