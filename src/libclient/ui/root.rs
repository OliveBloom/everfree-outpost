use std::prelude::v1::*;

use physics::v3::{V2, Region, Align};
use common::Gauge;

use Time;
use client::ClientObj;
use data::Data;
use debug::Debug as DebugDyn;
use hotbar::Hotbar;
use input::EventStatus;
use inventory::{Inventory, Inventories};
use misc::Misc;
use platform::Config;
use ui::geom::Geom;
use ui::input::{KeyAction, ActionEvent};
use ui::{dialog, dialogs, hotbar, debug, top_bar, inv_changes};
use ui::widget::*;

use ui::scroll_list;


pub struct Root {
    pub dialog: dialog::Dialog<dialogs::AnyDialog>,
    pub debug: debug::Debug,
    pub test_list: scroll_list::ScrollList,
}

impl Root {
    pub fn new() -> Root {
        Root {
            dialog: dialog::Dialog::new(dialogs::AnyDialog::none()),
            debug: debug::Debug::new(),
            test_list: scroll_list::ScrollList::new(V2::new(150, 100)),
        }
    }

    pub fn init<C: Config>(&mut self, cfg: &C) {
        self.debug.init(cfg);
    }
}

#[derive(Clone, Copy)]
pub struct RootDyn<'a> {
    pub screen_size: V2,
    pub now: Time,
    pub data: &'a Data,
    pub inventories: &'a Inventories,
    pub misc: &'a Misc,
    pub debug: &'a DebugDyn,
}

impl<'a> RootDyn<'a> {
    pub fn new(screen_size: (u16, u16),
               now: Time,
               data: &'a Data,
               inventories: &'a Inventories,
               misc: &'a Misc,
               debug: &'a DebugDyn) -> RootDyn<'a> {
        RootDyn {
            screen_size: V2::new(screen_size.0 as i32,
                                 screen_size.1 as i32),
            now: now,
            data: data,
            inventories: inventories,
            misc: misc,
            debug: debug,
        }
    }
}

impl<'a, 'b> Widget for WidgetPack<'a, Root, RootDyn<'b>> {
    fn size(&mut self) -> V2 {
        self.dyn.screen_size
    }

    fn walk_layout<V: Visitor>(&mut self, v: &mut V, pos: V2) {
        {
            // Hotbar
            let dyn = TopBarDyn::new(self.dyn.now,
                                     self.dyn.inventories.main_inventory(),
                                     &self.dyn.misc.hotbar,
                                     &self.dyn.misc.energy);
            let mut child = WidgetPack::stateless(top_bar::TopBar, &dyn);
            let rect = Region::sized(child.size()) + pos;
            v.visit(&mut child, rect);
        }

        {
            // Dialog
            let self_rect = Region::sized(self.size()) + pos;
            let dyn = dialogs::AnyDialogDyn::new(self.dyn.inventories, self.dyn.data);
            let mut child = WidgetPack::new(&mut self.state.dialog, &dyn);
            let child_rect = Region::sized(child.size());
            let rect = child_rect.align(self_rect, Align::Center, Align::Center);
            v.visit(&mut child, rect);
        }

        {
            // Debug pane
            let mut child = WidgetPack::new(&mut self.state.debug, &self.dyn.debug);
            let base = pos + V2::new(self.dyn.screen_size.x - child.size().x, 0);
            let rect = Region::sized(child.size()) + base;
            v.visit(&mut child, rect);
        }

        {
            // Inventory change display
            let dyn = inv_changes::InvChangesDyn::new(self.dyn.now,
                                                      self.dyn.data,
                                                      &self.dyn.misc.inv_changes);
            let mut child = WidgetPack::stateless(inv_changes::InvChanges, &dyn);
            let base = pos + self.dyn.screen_size - child.size();
            let rect = Region::sized(child.size()) + base;
            v.visit(&mut child, rect);
        }
    }

    fn render(&mut self, geom: &mut Geom, rect: Region<V2>) {
        use std::cell::Cell;
        use outpost_ui::geom::Point;
        use outpost_ui::widget::Widget;
        use outpost_ui::widgets::scroll::ScrollPane;
        use outpost_ui::widgets::text::Label;
        use ui2;
        use ui2::util::*;
        use ui2::widgets::list::TextList;

        let ui2_rect = rect.inset(200, 200, 150, 150);
        let mut ctx = ui2::context::Context::new(self.dyn.data, geom, ui2_rect);
        TextList {
            top: &Cell::new(0),
            focus: &Cell::new(0),
            items: &[
                "good morning",
                "everfree outpost",
                "!!!",
            ],
            size: Point { x: 300, y: 200 },
        }.on_paint(&mut ctx);
        /*
        let l = Label::new("hello, outpost");
        let st = Cell::new(0);
        ScrollPane::new(&st, from_v2(V2::new(300, 200)), l).on_paint(&mut ctx);
        */

    }

    fn on_key(&mut self, key: ActionEvent) -> EventStatus {
        use ui::dialogs::AnyDialog::{self, Inventory, Ability};

        let status = OnKeyVisitor::dispatch(self, key);
        if status.is_handled() {
            return status;
        }

        if let KeyAction::SetHotbar(idx) = key.code {
            // If the inventory or ability dialog is open, assign the selected item to the hotbar.
            let opt_assign =
                match self.state.dialog.inner {
                    Inventory(ref inv_dialog) => {
                        if let Some(inv) = self.dyn.inventories.main_inventory() {
                            Some((inv_dialog.focused_item(inv), false))
                        } else {
                            None
                        }
                    },

                    Ability(ref inv_dialog) => {
                        if let Some(inv) = self.dyn.inventories.ability_inventory() {
                            Some((inv_dialog.focused_item(inv), true))
                        } else {
                            None
                        }
                    },

                    _ => None
                };

            if let Some((item_id, is_ability)) = opt_assign {
                return EventStatus::Action(box move |c: &mut ClientObj| {
                    c.handle_hotbar_assign(idx as u8, item_id, is_ability);
                    c.handle_hotbar_select(idx as u8);
                });
            } else {
                // Select the indicated hotbar slot.
                return EventStatus::Action(box move |c: &mut ClientObj| {
                    c.handle_hotbar_select(idx as u8);
                });
            }
        }

        match self.state.dialog.inner {
            AnyDialog::None => EventStatus::Unhandled,
            _ => EventStatus::Handled,
        }
    }
}


#[derive(Clone, Copy)]
struct TopBarDyn<'a> {
    now: Time,
    inv: Option<&'a Inventory>,
    hotbar: &'a Hotbar,
    energy: &'a Gauge,
}

impl<'a> TopBarDyn<'a> {
    fn new(now: Time,
           inv: Option<&'a Inventory>,
           hotbar: &'a Hotbar,
           energy: &'a Gauge) -> TopBarDyn<'a> {
        TopBarDyn {
            now: now,
            inv: inv,
            hotbar: hotbar,
            energy: energy,
        }
    }
}

impl<'a> top_bar::TopBarDyn for TopBarDyn<'a> {
    fn hotbar_slot_info(&self, idx: u8) -> hotbar::SlotInfo {
        let inv = match self.inv {
            Some(x) => x,
            None => return hotbar::SlotInfo {
                item_id: 0,
                quantity: None,
                is_active_item: false,
                is_active_ability: false,
            },
        };
        let item_id = self.hotbar.item_id(idx);
        let quantity =
            if self.hotbar.is_item(idx) { Some(inv.count(item_id)) }
            else { None };
        let is_active_item = self.hotbar.active_item_index() == Some(idx);
        let is_active_ability = self.hotbar.active_ability_index() == Some(idx);

        hotbar::SlotInfo {
            item_id: item_id,
            quantity: quantity,
            is_active_item: is_active_item,
            is_active_ability: is_active_ability,
        }
    }

    fn cur_energy(&self) -> i32 {
        self.energy.get(self.now)
    }

    fn max_energy(&self) -> i32 {
        self.energy.max()
    }

    fn energy_tribe(&self) -> top_bar::Tribe {
        top_bar::Tribe::Earth
    }
}
