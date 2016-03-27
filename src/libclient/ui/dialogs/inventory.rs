use std::prelude::v1::*;
use physics::v3::{V2, scalar, Region};

use inventory::Item;
use ui::geom::Geom;
use ui::inventory;
use ui::widget::*;


pub struct Inventory {
    grid: inventory::Grid,
}

impl Inventory {
    pub fn new() -> Inventory {
        Inventory {
            grid: inventory::Grid::new(),
        }
    }
}

#[derive(Clone, Copy)]
pub struct InventoryDyn<'a> {
    inv: Option<&'a ::inventory::Inventory>,
}

impl<'a> InventoryDyn<'a> {
    pub fn new(inv: Option<&'a ::inventory::Inventory>) -> InventoryDyn<'a> {
        InventoryDyn {
            inv: inv,
        }
    }
}

impl<'a, 'b> Widget for WidgetPack<'a, Inventory, InventoryDyn<'b>> {
    fn size(&mut self) -> V2 {
        let mut child = WidgetPack::new(&mut self.state.grid, self.dyn);
        child.size()
    }

    fn walk_layout<V: Visitor>(&mut self, v: &mut V, pos: V2) {
        let mut child = WidgetPack::new(&mut self.state.grid, self.dyn);
        let rect = Region::sized(child.size()) + pos;
        v.visit(&mut child, rect);
    }

    fn render(&mut self, _geom: &mut Geom, _rect: Region<V2>) {
    }
}


impl<'a> inventory::GridDyn for InventoryDyn<'a> {
    fn grid_size(self) -> V2 {
        V2::new(6, 5)
    }

    fn len(self) -> usize {
        if let Some(inv) = self.inv {
            inv.len()
        } else {
            0
        }
    }

    fn item(self, i: usize) -> Item {
        self.inv.unwrap().items[i]
    }
}
