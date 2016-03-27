use std::prelude::v1::*;

use physics::v3::{V2, scalar, Region, Align};

use fonts::{self, FontMetricsExt};
use ui::atlas;
use ui::geom::Geom;
use ui::{dialog, hotbar};
use ui::widget::*;


#[derive(Clone, Copy)]
pub struct Root;

pub trait RootDyn: Copy {
    fn screen_size(self) -> V2;

    type DialogDyn: dialog::DialogDyn;
    fn dialog(self) -> Self::DialogDyn;

    type HotbarDyn: hotbar::HotbarDyn;
    fn hotbar(self) -> Self::HotbarDyn;
}

impl<D: RootDyn> Widget for WidgetPack<Root, D> {
    fn size(self) -> V2 {
        self.dyn.screen_size()
    }

    fn walk_layout<V: Visitor>(self, v: &mut V, pos: V2) {
        // Hotbar
        let child = WidgetPack::new(hotbar::Hotbar, self.dyn.hotbar());
        let child_size = child.size();
        let child_pos = pos + scalar(1);
        v.visit(child, Region::new(child_pos, child_pos + child_size));

        // Dialog
        let child = WidgetPack::new(dialog::Dialog, self.dyn.dialog());
        let child_rect = Region::sized(child.size());
        let self_rect = Region::sized(self.size()) + pos;
        v.visit(child, child_rect.align(self_rect, Align::Center, Align::Center));
    }

    fn render(self, _geom: &mut Geom, _rect: Region<V2>) {
    }
}
