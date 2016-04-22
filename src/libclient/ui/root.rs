use std::prelude::v1::*;

use physics::v3::{V2, scalar, Region, Align};

use client::ClientObj;
use data::Data;
use fonts::{self, FontMetricsExt};
use inventory::{Inventory, Inventories};
use misc;
use platform::Config;
use ui::atlas;
use ui::geom::Geom;
use ui::input::{KeyAction, EventStatus};
use ui::{dialog, dialogs, hotbar};
use ui::widget::*;


pub struct Root {
    pub dialog: dialog::Dialog<dialogs::AnyDialog>,
}

impl Root {
    pub fn new() -> Root {
        Root {
            dialog: dialog::Dialog::new(dialogs::AnyDialog::None),
        }
    }
}

#[derive(Clone, Copy)]
pub struct RootDyn<'a> {
    pub screen_size: V2,
    pub inventories: &'a Inventories,
    pub hotbar: &'a misc::Hotbar,
}

impl<'a> RootDyn<'a> {
    pub fn new(screen_size: V2,
               inventories: &'a Inventories,
               hotbar: &'a misc::Hotbar) -> RootDyn<'a> {
        RootDyn {
            screen_size: screen_size,
            inventories: inventories,
            hotbar: hotbar,
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
            let dyn = HotbarDyn::new(self.dyn.inventories.main_inventory(),
                                     self.dyn.hotbar);
            let mut child = WidgetPack::stateless(hotbar::Hotbar, dyn);
            let rect = Region::sized(child.size()) + pos + scalar(1);
            v.visit(&mut child, rect);
        }

        {
            // Dialog
            let self_rect = Region::sized(self.size()) + pos;
            let dyn = dialogs::AnyDialogDyn::new(self.dyn.inventories);
            let mut child = WidgetPack::new(&mut self.state.dialog, dyn);
            let child_rect = Region::sized(child.size());
            let rect = child_rect.align(self_rect, Align::Center, Align::Center);
            v.visit(&mut child, rect);
        }
    }

    fn render(&mut self, _geom: &mut Geom, _rect: Region<V2>) {
    }

    fn on_key(&mut self, key: KeyAction) -> EventStatus {
        use ui::dialogs::AnyDialog::{Inventory};

        let status = OnKeyVisitor::dispatch(self, key);
        if status.is_handled() {
            return status;
        }

        if let KeyAction::SetHotbar(idx) = key {
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

        EventStatus::Unhandled
    }
}


#[derive(Clone, Copy)]
struct HotbarDyn<'a> {
    inv: Option<&'a Inventory>,
    state: &'a misc::Hotbar,
}

impl<'a> HotbarDyn<'a> {
    fn new(inv: Option<&'a Inventory>,
           state: &'a misc::Hotbar) -> HotbarDyn<'a> {
        HotbarDyn {
            inv: inv,
            state: state,
        }
    }
}

impl<'a> hotbar::HotbarDyn for HotbarDyn<'a> {
    fn slot_info(self, idx: u8) -> hotbar::SlotInfo {
        let inv = match self.inv {
            Some(x) => x,
            None => return hotbar::SlotInfo {
                item_id: 0,
                quantity: None,
                is_active_item: false,
                is_active_ability: false,
            },
        };
        let item_id = self.state.item_id(idx);
        let quantity =
            if self.state.is_item(idx) { Some(inv.count(item_id)) }
            else { None };
        let is_active_item = self.state.active_item_index() == Some(idx);
        let is_active_ability = self.state.active_ability_index() == Some(idx);

        hotbar::SlotInfo {
            item_id: item_id,
            quantity: quantity,
            is_active_item: is_active_item,
            is_active_ability: is_active_ability,
        }
    }
}
