use std::prelude::v1::*;

use physics::v3::{V2, scalar, Region};

use super::WidgetBase;
use super::context::Context;


#[derive(Clone, Copy)]
pub struct ItemDisplay {
    base: WidgetBase,
}

pub trait ItemDyn {
    fn item_id(&self) -> u16;
    fn quantity(&self) -> Option<u16>;
}

impl ItemDisplay {
    pub fn new() -> ItemDisplay {
        ItemDisplay {
            base: WidgetBase::new(),
        }
    }

    // No iter_layout, because there are no children.

    pub fn calc_size(&self) -> V2 {
        scalar(16)
    }

    pub fn render<D: ItemDyn>(&self, ctx: &mut Context, dyn: D, pos: V2) {
        ctx.draw_item(dyn.item_id(), pos);
    }
}
