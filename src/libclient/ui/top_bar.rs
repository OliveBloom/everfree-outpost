use std::prelude::v1::*;

use physics::v3::{V2, scalar, Region, Align};

use client::ClientObj;
use fonts::{self, FontMetricsExt};
use ui::{Context, DragData};
use ui::atlas::{self, AtlasEntry};
use ui::geom::Geom;
use ui::input::EventStatus;
use ui::hotbar::{self, Hotbar, HotbarDyn};
use ui::widget::*;


pub struct TopBar;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tribe {
    Earth,
    Pegasus,
    Unicorn,
    Alicorn,
}

pub trait TopBarDyn: Copy {
    fn hotbar_slot_info(self, idx: u8) -> hotbar::SlotInfo;

    fn cur_energy(self) -> i32;
    fn max_energy(self) -> i32;
    fn energy_tribe(self) -> Tribe;
}

impl TopBar {
    pub fn size() -> V2 {
        let hotbar_size = Hotbar::size();
        let w = hotbar_size.x + 7 + 6;
        let h = 2 + atlas::ENERGY_BAR_CAP_LEFT.size().y;
        V2::new(w, h)
    }
}

impl<'a, D: TopBarDyn> Widget for WidgetPack<'a, TopBar, D> {
    fn size(&mut self) -> V2 { TopBar::size() }

    fn walk_layout<V: Visitor>(&mut self, v: &mut V, pos: V2) {
        let mut child = WidgetPack::stateless(Hotbar, HotbarDynWrapper(self.dyn));
        let rect = Region::sized(child.size()) + pos + V2::new(7, 1);
        v.visit(&mut child, rect);
    }

    fn render(&mut self, geom: &mut Geom, rect: Region<V2>) {
        geom.draw_ui(atlas::ENERGY_BAR_CAP_LEFT,
                     Region::sized(atlas::ENERGY_BAR_CAP_LEFT.size())
                         .align(rect, Align::Start, Align::End).min);
        geom.draw_ui(atlas::ENERGY_BAR_CAP_RIGHT,
                     Region::sized(atlas::ENERGY_BAR_CAP_RIGHT.size())
                         .align(rect, Align::End, Align::End).min);

        let x0 = rect.min.x + atlas::ENERGY_BAR_CAP_LEFT.size().x;
        let x1 = rect.max.x - atlas::ENERGY_BAR_CAP_RIGHT.size().x;
        let y0 = rect.min.y + 27;
        let y1 = y0 + atlas::ENERGY_BAR_BAR.size().y;

        // Bar can potentially fill from x0 + 1 to x1 - 1.
        let bar_len = (x1 - x0 - 2) * self.dyn.cur_energy() / self.dyn.max_energy();

        let (start_entry, mid_entry) = tribe_bar(self.dyn.energy_tribe());

        geom.draw_ui_tiled(atlas::ENERGY_BAR_BAR,
                           Region::new(V2::new(x0, y0), V2::new(x1, y1)));
        geom.draw_ui(atlas::ENERGY_BAR_START_E, V2::new(x0, y0 + 2));
        geom.draw_ui_tiled(atlas::ENERGY_BAR_MID_E,
                           Region::new(V2::new(x0 + 1, y0 + 2),
                                       V2::new(x0 + 1 + bar_len, y1 - 2)));

        let label = format!("{:3} / {:3}", self.dyn.cur_energy(), self.dyn.max_energy());
        let label_width = fonts::HOTBAR.measure_width(&label) as i32;
        let label_x = (x1 + x0 - label_width) / 2;
        geom.draw_str(&fonts::HOTBAR, &label, V2::new(label_x, y0 + 1));
    }
}


#[derive(Clone, Copy)]
struct HotbarDynWrapper<D>(D);

impl<D> HotbarDyn for HotbarDynWrapper<D> where D: TopBarDyn {
    fn slot_info(self, idx: u8) -> hotbar::SlotInfo {
        self.0.hotbar_slot_info(idx)
    }
}


fn tribe_bar(tribe: Tribe) -> (AtlasEntry, AtlasEntry) {
    match tribe {
        Tribe::Earth =>
            (atlas::ENERGY_BAR_START_E,
             atlas::ENERGY_BAR_MID_E),
        Tribe::Pegasus =>
            (atlas::ENERGY_BAR_START_P,
             atlas::ENERGY_BAR_MID_P),
        Tribe::Unicorn | Tribe::Alicorn =>
            (atlas::ENERGY_BAR_START_U,
             atlas::ENERGY_BAR_MID_U),
    }
}
