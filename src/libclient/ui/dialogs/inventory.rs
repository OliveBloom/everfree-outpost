use std::prelude::v1::*;
use types::*;
use common_proto::game::Request;
use physics::v3::{V2, Region};

use client::ClientObj;
use data::Data;
use input::EventStatus;
use inventory::Item;
use ui::Context;
use ui::atlas;
use ui::geom::Geom;
use ui::input::{KeyAction, ActionEvent};
use ui::inventory;
use ui::text;
use ui::util;
use ui::widget::*;


pub struct Inventory {
    grid: inventory::Grid,
    text: text::TextDisplay,
}

impl Inventory {
    pub fn new() -> Inventory {
        Inventory {
            grid: inventory::Grid::new(),
            text: text::TextDisplay::new(),
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
    data: &'a Data,
}

impl<'a> InventoryDyn<'a> {
    pub fn new(inv: Option<&'a ::inventory::Inventory>,
               data: &'a Data) -> InventoryDyn<'a> {
        InventoryDyn {
            inv: inv,
            data: data,
        }
    }

    fn as_grid_dyn(self) -> GridDyn<'a> {
        GridDyn::new(self.inv, true)
    }

    fn make_segments(&self, focus: usize) -> Vec<text::Segment<&str>> {
        let inv = match self.inv {
            Some(x) => x,
            None => return Vec::new(),
        };

        let item = match inv.items.get(focus) {
            Some(x) => x,
            None => return Vec::new(),
        };
        if item.id == NO_ITEM || item.quantity == 0 {
            return Vec::new();
        }

        let def = self.data.item_def(item.id);

        vec![
            text::Segment::Text(text::Attr::Bold, def.ui_name()),
            text::Segment::ParaBreak,
            text::Segment::Text(text::Attr::Normal, def.desc()),
        ]
    }
}

impl<'a, 'b> Widget for WidgetPack<'a, Inventory, InventoryDyn<'b>> {
    fn size(&mut self) -> V2 {
        let dyn = self.dyn.as_grid_dyn();
        let mut child = WidgetPack::new(&mut self.state.grid, &dyn);
        child.size() + V2::new(120 + 3, 0)
    }

    fn walk_layout<V: Visitor>(&mut self, v: &mut V, pos: V2) {
        let grid_size = {
            let dyn = self.dyn.as_grid_dyn();
            let mut child = WidgetPack::new(&mut self.state.grid, &dyn);
            let rect = Region::sized(child.size()) + pos;
            v.visit(&mut child, rect);
            rect.size()
        };

        let offset = V2::new(grid_size.x + 3, 0);
        self.state.text.size = V2::new(120, grid_size.y);

        let focus = self.state.grid.focus;
        let segments = self.dyn.make_segments(focus);

        let dyn = text::TextDisplayDyn::new(&segments);
        let mut child = WidgetPack::new(&mut self.state.text, &dyn);
        let rect = Region::sized(child.size()) + pos + offset;
        v.visit(&mut child, rect);
    }

    fn render(&mut self, _geom: &mut Geom, _rect: Region<V2>) {
    }
}


#[derive(Clone, Copy)]
pub struct GridDyn<'a> {
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
    fn grid_size(&self) -> V2 {
        V2::new(6, 5)
    }

    fn len(&self) -> usize {
        if let Some(inv) = self.inv {
            inv.len()
        } else {
            0
        }
    }

    fn item(&self, i: usize) -> Item {
        self.inv.unwrap().items[i]
    }

    fn active(&self) -> bool {
        self.active
    }

    fn inv_id(&self) -> Option<InventoryId> {
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

    pub fn on_close(self) -> EventStatus {
        EventStatus::Action(box move |c: &mut ClientObj| {
            c.platform().send_message(Request::CloseDialog(()));
        })
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
            let dyn = GridDyn::new(inv, idx as u8 == self.state.focus);
            let mut child = WidgetPack::new(&mut self.state.grid[idx], &dyn);
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

    fn on_key(&mut self, key: ActionEvent) -> EventStatus {
        let idx = self.state.focus as usize;
        let mut status = {
            let inv = self.dyn.invs.get(self.state.inv_id[idx]);
            let dyn = GridDyn::new(inv, true);
            let mut child = WidgetPack::new(&mut self.state.grid[idx], &dyn);
            child.on_key(key)
        };

        if !status.is_handled() {
            match key.code {
                KeyAction::MoveLeft if idx > 0 => {
                    self.state.focus -= 1;
                    status = EventStatus::Handled;
                },
                KeyAction::MoveRight if idx < 1 => {
                    self.state.focus += 1;
                    status = EventStatus::Handled;
                },
                KeyAction::Select => {
                    let src_inv = self.state.inv_id[idx];
                    let src_slot = self.state.grid[idx].focus as u8;
                    let dest_inv = self.state.inv_id[1 - idx];
                    let dest_slot = NO_SLOT;    // place items automatically

                    let amount = if key.shift() { 10 } else { 1 };

                    status = EventStatus::Action(box move |c: &mut ClientObj| {
                        c.platform().send_message(
                            Request::MoveItem(src_inv,
                                              src_slot,
                                              dest_inv,
                                              dest_slot,
                                              amount));
                    });
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
