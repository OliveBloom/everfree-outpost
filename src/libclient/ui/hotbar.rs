use std::prelude::v1::*;

use physics::v3::{V2, scalar, Region, Align};

use ui::atlas;
use ui::geom::Geom;
use ui::item::{ItemDisplay, ItemDyn};
use ui::widget::*;


#[derive(Clone, Copy)]
pub struct Slot;

pub trait SlotDyn: Copy {
    type ItemDyn: ItemDyn;
    fn item(self) -> Self::ItemDyn;
    fn color(self) -> u8;
}

impl Slot {
    pub fn size() -> V2 {
        ItemDisplay::size() + scalar(4 * 2)
    }
}

impl<D: SlotDyn> Widget for WidgetPack<Slot, D> {
    fn size(self) -> V2 { Slot::size() }

    fn walk_layout<V: Visitor>(self, v: &mut V, pos: V2) {
        let child = WidgetPack::new(ItemDisplay, self.dyn.item());
        let child_size = child.size();
        let child_pos = pos + scalar(4);
        v.visit(child, Region::new(child_pos, child_pos + child_size));
    }

    fn render(self, geom: &mut Geom, rect: Region<V2>) {
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
pub struct Hotbar;

pub trait HotbarDyn: Copy {
    type SlotDyn: SlotDyn;
    fn slot(self, i: usize) -> Self::SlotDyn;
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


impl<D: HotbarDyn> Widget for WidgetPack<Hotbar, D> {
    fn size(self) -> V2 { Hotbar::size() }

    fn walk_layout<V: Visitor>(self, v: &mut V, pos: V2) {
        let child_size = Slot::size();
        let base = pos + V2::new(0, 8);
        let step = V2::new(0, child_size.y + 1);

        for i in 0 .. 9 {
            let child = WidgetPack::new(Slot, self.dyn.slot(i));
            let child_pos = base + step * scalar(i as i32);
            v.visit(child, Region::new(child_pos, child_pos + child_size));
        }
    }

    fn render(self, geom: &mut Geom, rect: Region<V2>) {
        let (cap_w, cap_h) = atlas::HOTBAR_CAP_TOP.size;
        let cap_rect = Region::sized(V2::new(cap_w as i32, cap_h as i32));

        geom.draw_ui(atlas::HOTBAR_CAP_TOP,
                     cap_rect.align(rect, Align::Center, Align::Start).min);
        geom.draw_ui(atlas::HOTBAR_CAP_BOTTOM,
                     cap_rect.align(rect, Align::Center, Align::End).min);

        let bar_w = atlas::HOTBAR_BAR.size.0;
        let bar_rect = Region::sized(V2::new(bar_w as i32, rect.size().y - 2 * 7));

        let bar_dest = bar_rect.align(rect, Align::Center, Align::Center);
        geom.draw_ui_tiled(atlas::HOTBAR_BAR, bar_dest.min, bar_dest.size());
    }
}
