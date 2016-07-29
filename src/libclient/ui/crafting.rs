use std::prelude::v1::*;
use std::cmp;

use physics::v3::{V2, Vn, scalar, Region, Align};

use client::ClientObj;
use data::{RecipeDef, RecipeItem};
use inventory::Item;
use ui::{Context, DragData};
use ui::atlas;
use ui::geom::Geom;
use ui::input::{KeyEvent, EventStatus};
use ui::item;
use ui::widget::*;


// TODO: change these back to private
pub struct Arrow;

pub struct ArrowDyn {
    progress: u8,
}

impl Arrow {
    pub fn size() -> V2 {
        atlas::CRAFTING_ARROW_EMPTY.size()
    }
}

impl ArrowDyn {
    pub fn new(progress: u8) -> ArrowDyn {
        ArrowDyn {
            progress: progress,
        }
    }
}

impl<'a> Widget for WidgetPack<'a, Arrow, ArrowDyn> {
    fn size(&mut self) -> V2 { Arrow::size() }

    fn walk_layout<V: Visitor>(&mut self, v: &mut V, pos: V2) {}

    fn render(&mut self, geom: &mut Geom, rect: Region<V2>) {
        geom.draw_ui(atlas::CRAFTING_ARROW_EMPTY, rect.min);

        if self.dyn.progress > 0 {
            let width = self.dyn.progress as i32;
            let rect = Region::new(rect.min, V2::new(rect.min.x + width, rect.max.y));
            geom.draw_ui_tiled(atlas::CRAFTING_ARROW_FULL, rect);
        }
    }
}


pub struct Recipe;

pub struct RecipeDyn<'a> {
    recipe: &'a RecipeDef<'a>,
    progress: u8,
}

impl<'a> RecipeDyn<'a> {
    pub fn new(recipe: &'a RecipeDef, progress: u8) -> RecipeDyn<'a> {
        RecipeDyn {
            recipe: recipe,
            progress: progress,
        }
    }
}

impl Recipe {
    pub fn size() -> V2 {
        // Allow up to 2 columns of items on each side, with 1px margin on each side.
        let w = 4 * (item::ItemDisplay::size().x + 2) + 8 + atlas::CRAFTING_ARROW_EMPTY.size().x;
        // Allow up to 3 rows of items
        let h = 3 * (item::ItemDisplay::size().y + 2);
        V2::new(w, h)
    }
}

impl<'a, 'b> Widget for WidgetPack<'a, Recipe, RecipeDyn<'b>> {
    fn size(&mut self) -> V2 { Recipe::size() }

    fn walk_layout<V: Visitor>(&mut self, v: &mut V, pos: V2) {
        let rect = Region::sized(self.size()) + pos;

        // Handle the arrow first, so we can use closures for the items below.
        {
            let dyn = ArrowDyn::new(self.dyn.progress);
            let mut child = WidgetPack::stateless(Arrow, &dyn);
            let rect = Region::sized(child.size()).align(rect, Align::Center, Align::Center);
            v.visit(&mut child, rect);
        }


        let raw_step = item::ItemDisplay::size() + scalar(2);
        let step_y = raw_step.y;

        let mut do_col = |x, y, items: &[RecipeItem]| {
            // Apply margins
            let x = x + 1;
            let mut y = y + 1;

            for item in items {
                let dyn = item::ItemDyn::new(item.item, Some(item.quantity));
                let mut child = WidgetPack::stateless(item::ItemDisplay, &dyn);
                let rect = Region::sized(child.size()) + V2::new(x, y);
                v.visit(&mut child, rect);
                y += step_y;
            }
        };

        let mut do_side = |base: V2, step: V2, items: &[RecipeItem]| {
            if items.len() < 3 {
                // Display one column of 0-3 items
                let x = base.x + step.x;
                let y = base.y + (3 - items.len() as i32) * step.y / 2;
                do_col(x, y, items);

            } else if items.len() == 4 {
                // Display in a 2x2 grid
                let x = base.x;
                let y = base.y + step.y / 2;
                do_col(x + 0 * step.x, y, &items[0 .. 2]);
                do_col(x + 1 * step.x, y, &items[2 .. 4]);

            } else if items.len() <= 6 {
                // Display a column of 2-3 and a second column of 3.
                let x = base.x;
                let y0 = base.y + (6 - items.len() as i32) * step.y / 2;
                let y1 = base.y;
                let split = items.len() - 3;
                do_col(x + 0 * step.x, y0, &items[.. split]);
                do_col(x + 1 * step.x, y1, &items[split ..]);

            } else {
                assert!(false, "expected at most 6 items in recipe, but got {}", items.len());
            }
        };

        let left_pos = pos;
        do_side(left_pos, raw_step, self.dyn.recipe.inputs());

        let right_pos = V2::new(rect.max.x - raw_step.x,
                                rect.min.y);
        do_side(right_pos, raw_step * V2::new(-1, 1), self.dyn.recipe.outputs());
    }

    fn render(&mut self, geom: &mut Geom, rect: Region<V2>) {
        /*
        geom.draw_ui(atlas::CRAFTING_ARROW_EMPTY, rect.min);

        if self.dyn.progress > 0 {
            let width = self.dyn.progress as i32;
            let rect = Region::new(rect.min, V2::new(rect.min.x + width, rect.max.y));
            geom.draw_ui_tiled(atlas::CRAFTING_ARROW_FULL, rect);
        }
        */
    }
}




