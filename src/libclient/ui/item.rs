use std::prelude::v1::*;

use physics::v3::{V2, scalar, Region};

use ui::geom::Geom;
use ui::widget::*;


#[derive(Clone, Copy)]
pub struct ItemDisplay;

pub trait ItemDyn: Copy {
    fn item_id(self) -> u16;
    fn quantity(self) -> Option<u16>;
}

impl ItemDisplay {
    pub fn size() -> V2 { scalar(16) }
}

impl<D: ItemDyn> Widget for WidgetPack<ItemDisplay, D> {
    fn size(self) -> V2 { ItemDisplay::size() }

    fn walk_layout<V: Visitor>(self, v: &mut V, pos: V2) {
        // No children
    }

    fn render(self, geom: &mut Geom, rect: Region<V2>) {
        geom.draw_item(self.dyn.item_id(), rect.min);
    }
}
