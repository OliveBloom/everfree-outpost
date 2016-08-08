use std::prelude::v1::*;
use types::*;
use common_proto::game::Request;
use physics::v3::{V2, Region, Align};

use client::ClientObj;
use data::{Data, RecipeDef};
use input::EventStatus;
use ui::crafting;
use ui::dialogs;
use ui::geom::Geom;
use ui::input::{KeyAction, ActionEvent};
use ui::inventory;
use ui::scroll_list;
use ui::util;
use ui::widget::*;
use util::hash;


pub struct Crafting {
    inv_id: InventoryId,
    station_id: StructureId,
    template: u32,

    list: scroll_list::ScrollList,
    grid: inventory::Grid,

    cache: CraftingCache,
}

impl Crafting {
    pub fn new(inv_id: InventoryId, station_id: StructureId, template: u32) -> Crafting {
        Crafting {
            inv_id: inv_id,
            station_id: station_id,
            template: template,

            list: scroll_list::ScrollList::new(V2::new(LIST_WIDTH, 0)),
            grid: inventory::Grid::new(),

            cache: CraftingCache::new(),
        }
    }

    pub fn on_close(self) -> EventStatus {
        EventStatus::Action(box move |c: &mut ClientObj| {
            c.platform().send_message(Request::CloseDialog(()));
        })
    }
}

struct CraftingCache {
    abilities_hash: u64,
    template: u32,
    recipe_ids: Vec<u16>,
}

impl CraftingCache {
    fn new() -> CraftingCache {
        CraftingCache {
            abilities_hash: 0,
            template: 0,
            recipe_ids: Vec::new(),
        }
    }

    fn update(&mut self,
              dyn: &CraftingDyn,
              template: u32) {
        let abilities_hash =
            if let Some(abilities) = dyn.invs.ability_inventory() {
                hash(&abilities.items as &[_])
            } else {
                0
            };
        if abilities_hash == self.abilities_hash &&
           template == self.template {
            return;
        }

        self.abilities_hash = abilities_hash;
        self.template = template;
        self.recipe_ids = Vec::new();

        for id in 0 .. dyn.data.recipes().len() as u16 {
            let r = dyn.data.recipe(id);

            // Check ability filter
            if r.ability != 0 {
                if let Some(abilities) = dyn.invs.ability_inventory() {
                    if abilities.count(r.ability) == 0 {
                        continue;
                    }
                } else {
                    continue;
                }
            }

            // Check station filter
            if r.station != 0 && template != r.station {
                continue;
            }

            self.recipe_ids.push(id);
        }
    }
}

#[derive(Clone, Copy)]
pub struct CraftingDyn<'a> {
    invs: &'a ::inventory::Inventories,
    data: &'a Data,
}

impl<'a> CraftingDyn<'a> {
    pub fn new(invs: &'a ::inventory::Inventories,
               data: &'a Data) -> CraftingDyn<'a> {
        CraftingDyn {
            invs: invs,
            data: data,
        }
    }
}

const LIST_WIDTH: i32 = 120;

impl<'a, 'b> Widget for WidgetPack<'a, Crafting, CraftingDyn<'b>> {
    fn size(&mut self) -> V2 {
        util::size_from_children(self)
    }

    fn walk_layout<V: Visitor>(&mut self, v: &mut V, pos: V2) {
        self.state.cache.update(&self.dyn, self.state.template);


        let ox = LIST_WIDTH + 7;
        let mut oy = 0;

        let inv_rect = {
            let inv = self.dyn.invs.get(self.state.inv_id);
            let dyn = dialogs::inventory::GridDyn::new(inv, false);
            let mut child = WidgetPack::new(&mut self.state.grid, &dyn);
            let rect = Region::sized(child.size()) + pos + V2::new(ox, oy);
            v.visit(&mut child, rect);
            oy += rect.size().y;
            rect
        };

        oy += 7;

        if let Some(&id) = self.state.cache.recipe_ids.get(self.state.list.focus) {
            if (id as usize) < self.dyn.data.recipes().len() {
                let r = self.dyn.data.recipe(id);
                let dyn = crafting::RecipeDyn::new(&r, 0);
                let mut child = WidgetPack::stateless(crafting::Recipe, &dyn);
                let rect = Region::sized(child.size()) + pos + V2::new(ox, oy);
                let rect = rect.align(inv_rect, Align::Center, Align::None);
                v.visit(&mut child, rect);
            }
        }
        oy += crafting::Recipe::size().y;

        if self.state.list.size.y != oy {
            self.state.list.size.y = oy;
        }
        {
            let dyn = ListDyn {
                data: &self.dyn.data,
                recipe_ids: &self.state.cache.recipe_ids,
                inv: self.dyn.invs.get(self.state.inv_id),
            };
            let mut child = WidgetPack::new(&mut self.state.list, &dyn);
            let rect = Region::sized(child.size()) + pos + V2::new(0, 0);
            v.visit(&mut child, rect);
        }
    }

    fn render(&mut self, _geom: &mut Geom, _rect: Region<V2>) {
    }

    fn on_key(&mut self, key: ActionEvent) -> EventStatus {
        // TODO: make the inventory display "disabled", then use the normal key visitor first.
        let mut status = {
            let dyn = ListDyn {
                data: &self.dyn.data,
                recipe_ids: &self.state.cache.recipe_ids,
                inv: self.dyn.invs.get(self.state.inv_id),
            };
            let mut child = WidgetPack::new(&mut self.state.list, &dyn);
            child.on_key(key)
        };

        if !status.is_handled() {
            match key.code {
                KeyAction::Select => {
                    let inv_id = self.state.inv_id;
                    let station_id = self.state.station_id;
                    let recipe_id = self.state.cache.recipe_ids[self.state.list.focus];
                    let count = if key.shift() { 10 } else { 1 };

                    status = EventStatus::Action(box move |c: &mut ClientObj| {
                        c.platform().send_message(
                            Request::CraftRecipe(station_id,
                                                 inv_id,
                                                 recipe_id,
                                                 count));
                    });
                },
                _ => {},
            }
        }

        status
    }
}

struct ListDyn<'a> {
    data: &'a Data,
    recipe_ids: &'a [u16],
    inv: Option<&'a ::inventory::Inventory>,
}

impl<'a> ListDyn<'a> {
    fn get_recipe(&self, idx: usize) -> Option<RecipeDef<'a>> {
        let id = self.recipe_ids[idx];
        if (id as usize) < self.data.recipes().len() {
            Some(self.data.recipe(id))
        } else {
            None
        }
    }
}

impl<'a> scroll_list::ScrollListDyn for ListDyn<'a> {
    fn get(&self, idx: usize) -> &str {
        self.get_recipe(idx).map_or("[unknown recipe]", |r| r.ui_name())
    }

    fn is_enabled(&self, idx: usize) -> bool {
        if let Some(recipe) = self.get_recipe(idx) {
            if let Some(inv) = self.inv {
                if recipe.inputs().iter()
                         .all(|input| inv.count(input.item) >= input.quantity) {
                    return true;
                }
            }
        }
        false
    }

    fn len(&self) -> usize {
        self.recipe_ids.len()
    }
}
