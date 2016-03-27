use std::prelude::v1::*;

use physics::v3::{V2, scalar, Region, Align};

use fonts::{self, FontMetricsExt};
use ui::atlas;
use ui::geom::Geom;
use ui::dyn;
use ui::{dialog, hotbar};
use ui::widget::*;


pub struct Root {
    pub hotbar: hotbar::Hotbar,
    pub dialog: dialog::Dialog<dyn::DialogInner>,
}

impl Root {
    pub fn new() -> Root {
        Root {
            hotbar: hotbar::Hotbar::new(),
            dialog: dialog::Dialog::new(dyn::DialogInner::None),
        }
    }
}

pub trait RootDyn: Copy {
    fn screen_size(self) -> V2;

    /*
    type DialogDyn: dyn::DialogInnerDyn;
    fn dialog(self) -> Self::DialogDyn;
    */

    type HotbarDyn: hotbar::HotbarDyn;
    fn hotbar(self) -> Self::HotbarDyn;
}

impl<'a, D: RootDyn> Widget for WidgetPack<'a, Root, D> {
    fn size(&mut self) -> V2 {
        self.dyn.screen_size()
    }

    fn walk_layout<V: Visitor>(&mut self, v: &mut V, pos: V2) {
        {
            // Hotbar
            let mut child = WidgetPack::new(&mut self.state.hotbar, self.dyn.hotbar());
            let child_rect = Region::sized(child.size()) + pos + scalar(1);
            v.visit(&mut child, child_rect);
        }

        /*
        {
            // Dialog
            let mut child = WidgetPack::new(&mut self.state.dialog, self.dyn.dialog());
            let child_rect = Region::sized(child.size());
            let self_rect = Region::sized(self.size()) + pos;
            v.visit(&mut child, child_rect.align(self_rect, Align::Center, Align::Center));
        }
        */
    }

    fn render(&mut self, _geom: &mut Geom, _rect: Region<V2>) {
    }
}
