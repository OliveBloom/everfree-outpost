use std::prelude::v1::*;

use physics::v3::{V2, scalar, Region, Align};

use data::Data;
use platform::{Config, ConfigKey};
use ui::atlas;
use ui::geom::Geom;
use ui::item;
use ui::widget::*;


#[derive(Clone, Copy)]
struct Slot;

#[derive(Clone, Copy)]
struct SlotDyn {
    item_dyn: item::ItemDyn,
    color: u8,
}

impl Slot {
    pub fn size() -> V2 {
        item::ItemDisplay::size() + scalar(4 * 2)
    }
}

impl<'a> Widget for WidgetPack<'a, Slot, SlotDyn> {
    fn size(&mut self) -> V2 { Slot::size() }

    fn walk_layout<V: Visitor>(&mut self, v: &mut V, pos: V2) {
        let mut child = WidgetPack::stateless(item::ItemDisplay, self.dyn.item_dyn);
        let rect = Region::sized(child.size()) + pos + scalar(4);
        v.visit(&mut child, rect);
    }

    fn render(&mut self, geom: &mut Geom, rect: Region<V2>) {
        let bg = match self.dyn.color {
            0 => atlas::HOTBAR_BOX_YELLOW,
            1 => atlas::HOTBAR_BOX_GREEN,
            2 => atlas::HOTBAR_BOX_BLUE,
            3 => atlas::HOTBAR_BOX_RED,
            _ => unreachable!(),
        };

        geom.draw_ui(bg, rect.min);
    }
}


#[derive(Clone, Copy)]
pub struct HotbarSlot {
    pub item_id: u16,
    pub is_ability: bool,
}

pub struct Hotbar {
    pub slots: [HotbarSlot; 9],
    pub cur_item: i8,
    pub cur_ability: i8,
}

impl Hotbar {
    pub fn new() -> Hotbar {
        Hotbar {
            slots: [HotbarSlot { item_id: 0, is_ability: false }; 9],
            cur_item: -1,
            cur_ability: -1,
        }
    }

    pub fn init<C: Config>(&mut self, cfg: &C, data: &Data) {
        for i in 0 .. 9 {
            let name = cfg.get_str(ConfigKey::HotbarItemName(i as u8));
            let id = match data.find_item_id(&name) {
                Some(x) => x,
                None => continue,
            };
            let is_item = cfg.get_int(ConfigKey::HotbarIsItem(i as u8)) != 0;
            self.slots[i].item_id = id;
            self.slots[i].is_ability = !is_item;
        }

        self.cur_item = cfg.get_int(ConfigKey::HotbarActiveItem) as i8;
        self.cur_ability = cfg.get_int(ConfigKey::HotbarActiveAbility) as i8;
    }

    pub fn set_slot(&mut self, idx: i8, item_id: u16, is_ability: bool) {
        if idx < 0 || idx >= 9 {
            return;
        }

        self.slots[idx as usize] = HotbarSlot {
            item_id: item_id,
            is_ability: is_ability,
        };

        if !is_ability {
            // Replaced the selected ability with an item
            if self.cur_ability == idx {
                self.cur_ability = -1;
            }
        } else {
            // Vice versa
            if self.cur_item == idx {
                self.cur_item = -1;
            }
        }
    }

    pub fn select(&mut self, idx: i8) {
        if idx < 0 || idx >= 9 {
            return;
        }

        if !self.slots[idx as usize].is_ability {
            self.cur_item = idx;
        } else {
            self.cur_ability = idx;
        }
    }
}

pub trait HotbarDyn: Copy {
    fn item_count(self, item_id: u16) -> u16;
}

impl Hotbar {
    pub fn size() -> V2 {
        let w = Slot::size().x;

        // Height of all slots, including the gaps in between.
        let slot_h = 9 * Slot::size().y + 8 * 1;
        let h = 8 + slot_h + 8;

        V2::new(w, h)
    }
}

impl<'a, D: HotbarDyn> Widget for WidgetPack<'a, Hotbar, D> {
    fn size(&mut self) -> V2 { Hotbar::size() }

    fn walk_layout<V: Visitor>(&mut self, v: &mut V, pos: V2) {
        let base = pos + V2::new(0, 8);
        let child_size = Slot::size();
        let step = V2::new(0, child_size.y + 1);

        for i in 0 .. 9 {
            let slot = self.state.slots[i];
            let qty =
                if slot.is_ability || slot.item_id == 0 { None }
                else { Some(self.dyn.item_count(slot.item_id)) };
            let dyn = SlotDyn {
                item_dyn: item::ItemDyn::new(slot.item_id, qty),
                color:
                    if i as i8 == self.state.cur_item { 1 }
                    else if i as i8 == self.state.cur_ability { 1 }
                    else { 0 },
            };

            let mut child = WidgetPack::stateless(Slot, dyn);
            let rect = Region::sized(child_size) + base + step * scalar(i as i32);
            v.visit(&mut child, rect);
        }
    }

    fn render(&mut self, geom: &mut Geom, rect: Region<V2>) {
        let (cap_w, cap_h) = atlas::HOTBAR_CAP_TOP.size;
        let cap_rect = Region::sized(V2::new(cap_w as i32, cap_h as i32));

        geom.draw_ui(atlas::HOTBAR_CAP_TOP,
                     cap_rect.align(rect, Align::Center, Align::Start).min);
        geom.draw_ui(atlas::HOTBAR_CAP_BOTTOM,
                     cap_rect.align(rect, Align::Center, Align::End).min);

        let bar_w = atlas::HOTBAR_BAR.size.0;
        let bar_rect = Region::sized(V2::new(bar_w as i32, rect.size().y - 2 * 7));

        let bar_dest = bar_rect.align(rect, Align::Center, Align::Center);
        geom.draw_ui_tiled(atlas::HOTBAR_BAR, bar_dest);
    }
}
