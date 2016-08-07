use types::*;
use physics::v3::{V2, Region};

use data::Data;
use fonts::{self, FontMetricsExt};
use inv_changes;
use ui::geom::Geom;
use ui::item;
use ui::widget::*;


const WIDTH: i32 = 120;
// ItemDisplay is 16px, so that's enough for a quantity_string.  Then add 4px for the +/-.
const COUNT_WIDTH: i32 = 20;

#[derive(Clone, Copy)]
struct Entry;

struct EntryDyn<'a> {
    entry: &'a inv_changes::Entry,
    data: &'a Data,
}

impl Entry {
    pub fn size() -> V2 {
        // Item icon height + 1px padding on each side
        V2::new(WIDTH, 16 + 2)
    }
}

impl<'a> Widget for WidgetPack<'a, Entry, EntryDyn<'a>> {
    fn size(&mut self) -> V2 { Entry::size() }

    fn walk_layout<V: Visitor>(&mut self, _v: &mut V, _pos: V2) {}

    fn render(&mut self, geom: &mut Geom, rect: Region<V2>) {
        let text_y = (self.size().y - fonts::NAME.height as i32) / 2;

        // Draw the quantity
        {
            let count = self.dyn.entry.count;
            let sign = if count > 0 { '+' } else { '-' };
            let s = format!("{}{}", sign, item::quantity_string(count.abs() as u16));
            let width = fonts::NAME.measure_width(&s) as i32;
            let pos = V2::new(COUNT_WIDTH - width, text_y);
            geom.draw_str(&fonts::NAME, &s, rect.min + pos);
        }

        // Draw the item
        {
            let pos = V2::new(COUNT_WIDTH + 1, 1);
            geom.draw_item(self.dyn.entry.item, rect.min + pos);
        }

        // Draw the name
        {
            let name = self.dyn.data.item_def(self.dyn.entry.item).ui_name();
            let pos = V2::new(COUNT_WIDTH + 16 + 2, text_y);
            geom.draw_str(&fonts::NAME, name, rect.min + pos);
        }
    }
}


pub struct InvChanges;

pub struct InvChangesDyn<'a> {
    now: Time,
    data: &'a Data,
    inv_changes: &'a inv_changes::InvChanges,
}

impl<'a> InvChangesDyn<'a> {
    pub fn new(now: Time,
               data: &'a Data,
               inv_changes: &'a inv_changes::InvChanges) -> InvChangesDyn<'a> {
        InvChangesDyn {
            now: now,
            data: data,
            inv_changes: inv_changes,
        }
    }
}

impl<'a> Widget for WidgetPack<'a, InvChanges, InvChangesDyn<'a>> {
    fn size(&mut self) -> V2 {
        let s = Entry::size();
        V2::new(s.x, s.y * self.dyn.inv_changes.len() as i32)
    }

    fn walk_layout<V: Visitor>(&mut self, v: &mut V, pos: V2) {
        let size = Entry::size();
        for (i, entry) in self.dyn.inv_changes.iter().enumerate() {
            if entry.time < self.dyn.now - inv_changes::DISPLAY_TIME {
                continue;
            }

            let dyn = EntryDyn { entry: entry, data: self.dyn.data };
            let mut child = WidgetPack::stateless(Entry, &dyn);
            let rect = Region::sized(size) + pos + V2::new(0, size.y * i as i32);
            v.visit(&mut child, rect);
        }
    }

    fn render(&mut self, _geom: &mut Geom, _rect: Region<V2>) {}
}
