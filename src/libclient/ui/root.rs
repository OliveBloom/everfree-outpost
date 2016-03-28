use std::prelude::v1::*;

use physics::v3::{V2, scalar, Region, Align};

use fonts::{self, FontMetricsExt};
use inventory::{Inventory, Inventories};
use ui::atlas;
use ui::geom::Geom;
use ui::input::KeyAction;
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
            dialog: dialog::Dialog::new(dialogs::AnyDialog::Inventory(dialogs::Inventory::new())),
        }
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

    fn on_key(&mut self, key: KeyAction) -> bool {
        use ui::dialogs::AnyDialog::*;

        if OnKeyVisitor::dispatch(self, key) {
            return true;
        }

        if let KeyAction::SetHotbar(idx) = key {
            // If the inventory or ability dialog is open, assign the selected item to the hotbar.
            match self.state.dialog.inner {
                Inventory(ref inv_dialog) => {
                    if let Some(inv) = self.dyn.inventories.main_inventory() {
                        let item_id = inv_dialog.focused_item(inv);
                        self.state.hotbar.set_slot(idx, item_id, false);
                    }
                },
                _ => {},
            }

            // Select the indicated hotbar slot.
            self.state.hotbar.select(idx);

            return true;
        }

        false
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
