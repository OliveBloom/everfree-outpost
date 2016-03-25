use std::prelude::v1::*;

use physics::v3::{V2, scalar, Region};

use super::WidgetBase;
use super::context::Context;
use super::atlas;
use super::item::{ItemDisplay, ItemDyn};


#[derive(Clone, Copy)]
pub struct Slot {
    base: WidgetBase,

    item: ItemDisplay,
}

pub trait SlotDyn {
    type ItemDyn: ItemDyn;
    fn item(&self) -> Self::ItemDyn;
    fn color(&self) -> u8;
}

impl Slot {
    pub fn new() -> Slot {
        Slot {
            base: WidgetBase::new(),
            item: ItemDisplay::new(),
        }
    }

    pub fn calc_size(&self) -> V2 {
        self.item.calc_size() + scalar(4 * 2)
    }

    pub fn iter_layout<F>(&self, pos: V2, mut f: F)
            where F: FnMut(&ItemDisplay, Region<V2>) {
        let size = self.item.calc_size();
        let bounds = Region::new(scalar(0), size) + scalar(4);
        f(&self.item, bounds + pos);
    }

    pub fn render<D: SlotDyn>(&self, ctx: &mut Context, dyn: D, pos: V2) {
        let bg = match dyn.color() {
            0 => atlas::HOTBAR_BOX_YELLOW,
            1 => atlas::HOTBAR_BOX_GREEN,
            2 => atlas::HOTBAR_BOX_BLUE,
            3 => atlas::HOTBAR_BOX_RED,
            _ => unreachable!(),
        };
        ctx.draw_ui(bg, pos);

        self.iter_layout(pos, |item, bounds| {
            item.render(ctx, dyn.item(), bounds.min);
        });
    }
}


#[derive(Clone, Copy)]
pub struct Hotbar {
    base: WidgetBase,

    slots: [Slot; 9],
}

pub trait HotbarDyn {
    type SlotDyn: SlotDyn;
    fn slot(&self, i: usize) -> Self::SlotDyn;
}

impl Hotbar {
    pub fn new() -> Hotbar {
        Hotbar {
            base: WidgetBase::new(),
            slots: [Slot::new(); 9],
        }
    }

    pub fn calc_size(&self) -> V2 {
        let slot_size = self.slots[0].calc_size();
        let w = slot_size.x;
        // Height of all slots, including the gaps in between.
        let h_slots = 9 * slot_size.y + 8 * 1;
        let h = 8 + h_slots + 8;

        V2::new(w, h)
    }

    pub fn iter_layout<F>(&self, pos: V2, mut f: F)
            where F: FnMut(&Slot, Region<V2>) {
        let base = 8;
        let spacing = self.slots[0].calc_size().y + 1;
        for (i, slot) in self.slots.iter().enumerate() {
            let size = slot.calc_size();
            let offset = V2::new(0, base + spacing * i as i32);
            f(slot, Region::new(offset, offset + size) + pos);
        }
    }

    pub fn render<D: HotbarDyn>(&self, ctx: &mut Context, dyn: D, pos: V2) {
        let size = self.calc_size();
        let w = size.x;
        let h = size.y;

        let (cap_w, cap_h) = atlas::HOTBAR_CAP_TOP.size;
        let cap_x = (w - cap_w as i32) / 2;
        ctx.draw_ui(atlas::HOTBAR_CAP_TOP, pos + V2::new(cap_x, 0));
        ctx.draw_ui(atlas::HOTBAR_CAP_BOTTOM, pos + V2::new(cap_x, h - cap_h as i32));

        let bar_w = atlas::HOTBAR_BAR.size.0;
        let bar_x = (w - bar_w as i32) / 2;
        ctx.draw_ui_tiled(atlas::HOTBAR_BAR,
                          pos + V2::new(bar_x, cap_h as i32),
                          V2::new(bar_w as i32, h - 2 * cap_h as i32));

        let mut iter = 0..;
        self.iter_layout(pos, |slot, bounds| {
            let i = iter.next().unwrap();
            slot.render(ctx, dyn.slot(i), bounds.min);
        });
    }
}
