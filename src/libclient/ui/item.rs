use std::prelude::v1::*;

use physics::v3::{V2, scalar, Region};

use fonts::{self, FontMetricsExt};
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

impl<'a, D: ItemDyn> Widget for WidgetPack<'a, ItemDisplay, D> {
    fn size(&mut self) -> V2 { ItemDisplay::size() }

    fn walk_layout<V: Visitor>(&mut self, v: &mut V, pos: V2) {
        // No children
    }

    fn render(&mut self, geom: &mut Geom, rect: Region<V2>) {
        geom.draw_item(self.dyn.item_id(), rect.min);
        if let Some(qty) = self.dyn.quantity() {
            let s = quantity_string(qty);
            let width = fonts::HOTBAR.measure_width(&s);
            let offset = V2::new(width as i32, fonts::HOTBAR.height as i32);
            geom.draw_str(&fonts::HOTBAR, &s, rect.max - offset + scalar(1));
        }
    }
}

fn quantity_string(quantity: u16) -> String {
    if quantity < 1000 {
        format!("{}", quantity)
    } else if quantity < 10000 {
        let frac = quantity / 100 % 10;
        let whole = quantity / 1000;
        format!("{}.{}k", whole, frac)
    } else {
        let thousands = quantity / 1000;
        format!("{}k", thousands)
    }
}
