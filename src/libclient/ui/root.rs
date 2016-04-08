use std::prelude::v1::*;

use physics::v3::{V2, scalar, Region, Align};

use client::ClientObj;
use data::Data;
use fonts::{self, FontMetricsExt};
use inventory::{Inventory, Inventories};
use platform::Config;
use ui::atlas;
use ui::geom::Geom;
use ui::input::{KeyAction, EventStatus};
use ui::{dialog, dialogs, hotbar};
use ui::widget::*;


pub struct Root {
    pub hotbar: hotbar::Hotbar,
    pub dialog: dialog::Dialog<dialogs::AnyDialog>,
}

impl Root {
    pub fn new() -> Root {
        Root {
            hotbar: hotbar::Hotbar::new(),
            dialog: dialog::Dialog::new(dialogs::AnyDialog::None),
        }
    }

    pub fn init_hotbar<C: Config>(&mut self, cfg: &C, data: &Data) {
        self.hotbar.init(cfg, data);
    }

    fn hotbar_assign(&mut self, idx: i8, item_id: u16, is_ability: bool) -> EventStatus {
        self.hotbar.set_slot(idx, item_id, is_ability);
        self.hotbar.select(idx);

        return EventStatus::Action(box move |c: &mut ClientObj| {
            let name = String::from(c.data().item_def(item_id).name());
            let platform = c.platform();
            let cfg = platform.config_mut();
            hotbar::Hotbar::config_set_slot(cfg, idx, name, is_ability);
            hotbar::Hotbar::config_select(cfg, idx, is_ability);
        });
    }
}

#[derive(Clone, Copy)]
pub struct RootDyn<'a> {
    pub screen_size: V2,
    pub inventories: &'a Inventories,
}

impl<'a> RootDyn<'a> {
    pub fn new(screen_size: V2,
               inventories: &'a Inventories) -> RootDyn<'a> {
        RootDyn {
            screen_size: screen_size,
            inventories: inventories,
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
            let dyn = HotbarDyn::new(self.dyn.inventories.main_inventory());
            let mut child = WidgetPack::new(&mut self.state.hotbar, dyn);
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
                return self.state.hotbar_assign(idx, item_id, is_ability);
            } else {
                // Select the indicated hotbar slot.
                let is_ability = self.state.hotbar.slots[idx as usize].is_ability;
                self.state.hotbar.select(idx);
                return EventStatus::Action(box move |c: &mut ClientObj| {
                    hotbar::Hotbar::config_select(c.platform().config_mut(), idx, is_ability);
                });
            }
        }

        EventStatus::Unhandled
    }
}


#[derive(Clone, Copy)]
struct HotbarDyn<'a> {
    inv: Option<&'a Inventory>,
}

impl<'a> HotbarDyn<'a> {
    fn new(inv: Option<&'a Inventory>) -> HotbarDyn<'a> {
        HotbarDyn {
            inv: inv,
        }
    }
}

impl<'a> hotbar::HotbarDyn for HotbarDyn<'a> {
    fn item_count(self, item_id: u16) -> u16 {
        if let Some(inv) = self.inv {
            inv.count(item_id)
        } else {
            0
        }
    }
}
