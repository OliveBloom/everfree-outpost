use std::prelude::v1::*;
use std::cmp;
use physics::v3::{V2, scalar, Region};

use inventory::{Item, InventoryId};
use ui::Context;
use ui::atlas;
use ui::geom::Geom;
use ui::input::{KeyAction, EventStatus};
use ui::inventory;
use ui::util;
use ui::widget::*;


pub struct Inventory {
    grid: inventory::Grid,
    ability: bool,
}

impl Inventory {
    pub fn new(ability: bool) -> Inventory {
        Inventory {
            grid: inventory::Grid::new(),
            ability: ability,
        }
    }

    pub fn focused_item(&self, inv: &::inventory::Inventory) -> u16 {
        let idx = self.grid.focus;
        if idx < inv.len() {
            inv.items[idx].id
        } else {
            0
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

    fn as_grid_dyn(self) -> GridDyn<'a> {
        GridDyn::new(self.inv, true)
    }
}

impl<'a, 'b> Widget for WidgetPack<'a, Inventory, InventoryDyn<'b>> {
    fn size(&mut self) -> V2 {
        let mut child = WidgetPack::new(&mut self.state.grid, self.dyn.as_grid_dyn());
        child.size()
    }

    fn walk_layout<V: Visitor>(&mut self, v: &mut V, pos: V2) {
        let mut child = WidgetPack::new(&mut self.state.grid, self.dyn.as_grid_dyn());
        let rect = Region::sized(child.size()) + pos;
        v.visit(&mut child, rect);
    }

    fn render(&mut self, _geom: &mut Geom, _rect: Region<V2>) {
    }
}


#[derive(Clone, Copy)]
struct GridDyn<'a> {
    inv: Option<&'a ::inventory::Inventory>,
    active: bool,
}

impl<'a> GridDyn<'a> {
    pub fn new(inv: Option<&'a ::inventory::Inventory>,
               active: bool) -> GridDyn<'a> {
        GridDyn {
            inv: inv,
            active: active,
        }
    }
}

impl<'a> inventory::GridDyn for GridDyn<'a> {
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

    fn active(self) -> bool {
        self.active
    }

    fn inv_id(self) -> Option<u32> {
        self.inv.map(|i| i.id)
    }
}


pub struct Container {
    inv_id: [InventoryId; 2],
    grid: [inventory::Grid; 2],
    focus: u8,
}

impl Container {
    pub fn new(inv_id1: InventoryId, inv_id2: InventoryId) -> Container {
        Container {
            inv_id: [inv_id1,
                     inv_id2],
            grid: [inventory::Grid::new(),
                   inventory::Grid::new()],
            focus: 1,
        }
    }
}

#[derive(Clone, Copy)]
pub struct ContainerDyn<'a> {
    invs: &'a ::inventory::Inventories,
}

impl<'a> ContainerDyn<'a> {
    pub fn new(invs: &'a ::inventory::Inventories) -> ContainerDyn<'a> {
        ContainerDyn {
            invs: invs,
        }
    }
}

impl<'a, 'b> Widget for WidgetPack<'a, Container, ContainerDyn<'b>> {
    fn size(&mut self) -> V2 {
        util::size_from_children(self)
    }

    fn walk_layout<V: Visitor>(&mut self, v: &mut V, pos: V2) {
        let mut x = 0;
        for idx in 0 .. 2 {
            let inv = self.dyn.invs.get(self.state.inv_id[idx]);
            let mut child = WidgetPack::new(&mut self.state.grid[idx],
                                            GridDyn::new(inv, idx as u8 == self.state.focus));
            let rect = Region::sized(child.size()) + pos + V2::new(x, 0);
            v.visit(&mut child, rect);
            x += rect.size().x + 7;
        }
    }

    fn render(&mut self, geom: &mut Geom, rect: Region<V2>) {
        let mut i = 0;
        let top = rect.min.y + 8;
        let bottom = rect.max.y - 8;
        util::RectVisitor::dispatch(self, |r| {
            if i < 1 {
                let x = rect.min.x + r.max.x + 2;
                geom.draw_ui_tiled(atlas::SEPARATOR_VERT,
                                   Region::new(V2::new(x, top), V2::new(x + 3, bottom)));
                geom.draw_ui(atlas::SEPARATOR_CAP_N, V2::new(x, top - 1));
                geom.draw_ui(atlas::SEPARATOR_CAP_S, V2::new(x, bottom));
            }
            i += 1;
        });
    }

    fn on_key(&mut self, key: KeyAction) -> EventStatus {
        let idx = self.state.focus as usize;
        let inv = self.dyn.invs.get(self.state.inv_id[idx]);
        let mut child = WidgetPack::new(&mut self.state.grid[idx],
                                        GridDyn::new(inv, true));
        let mut status = child.on_key(key);

        if !status.is_handled() {
            match key {
                KeyAction::MoveLeft(_) if idx > 0 => {
                    self.state.focus -= 1;
                    status = EventStatus::Handled;
                },
                KeyAction::MoveRight(_) if idx < 1 => {
                    self.state.focus += 1;
                    status = EventStatus::Handled;
                },
                _ => {},
            }
        }

        status
    }

    fn on_mouse_move(&mut self, ctx: &mut Context, rect: Region<V2>) -> EventStatus {
        let mut i = 0;
        let mut hit = None;
        let pos = ctx.mouse_pos - rect.min;
        util::RectVisitor::dispatch(self, |r| {
            if r.contains(pos) {
                hit = Some(i);
            }
            i += 1;
        });

        if let Some(idx) = hit {
            self.state.focus = idx;
        }

        MouseEventVisitor::dispatch(MouseEvent::Move, self, ctx, rect)
    }
}
