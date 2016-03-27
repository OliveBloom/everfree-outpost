use std::prelude::v1::*;

use physics::v3::{V2, scalar, Region, Align};

use fonts::{self, FontMetricsExt};
use ui::atlas;
use ui::geom::Geom;
use ui::inventory;
use ui::widget::*;


#[derive(Clone, Copy)]
pub struct Dialog;

pub trait DialogDyn: Copy {
    type InvDyn: inventory::GridDyn;

    fn inv_grid(self) -> Self::InvDyn;
    fn with_title<R, F: FnOnce(&str) -> R>(self, f: F) -> R;
}

#[inline]
fn extra_height() -> i32 {
    atlas::DIALOG_TITLE_CENTER.size().y + 3
}

impl<D: DialogDyn> Widget for WidgetPack<Dialog, D> {
    fn size(self) -> V2 {
        let body = WidgetPack::new(inventory::Grid(6), self.dyn.inv_grid());
        let base_size = body.size() + scalar(6 * 2);
        base_size + V2::new(0, extra_height())
    }

    fn walk_layout<V: Visitor>(self, v: &mut V, pos: V2) {
        let child = WidgetPack::new(inventory::Grid(6), self.dyn.inv_grid());
        let child_size = child.size();
        let child_pos = pos + scalar(6) + V2::new(0, extra_height());
        v.visit(child, Region::new(child_pos, child_pos + child_size));
    }

    fn render(self, geom: &mut Geom, rect: Region<V2>) {
        let lower = Region::new(rect.min + V2::new(0, extra_height()), rect.max);

        let title_height = atlas::DIALOG_TITLE_CENTER.size().y;
        let title = Region::new(rect.min, V2::new(rect.max.x, rect.min.y + title_height));

        // We need to place a whole number of spacers, centered, in the eligible area.
        let spacer_size = atlas::DIALOG_SPACER.size();
        let spacer_room = title.size().x - 2 * DIALOG_SPACER_INSET;
        let spacer_count = spacer_room / spacer_size.x;
        let spacer_width = spacer_count * spacer_size.x;
        let spacer =
            Region::sized(V2::new(spacer_width, spacer_size.y))
                .align(title, Align::Center, Align::End) + V2::new(0, 3);


        // Spacer has to be drawn first so it appears on bottom.
        geom.draw_ui_tiled(atlas::DIALOG_SPACER, spacer);


        let n = atlas::DIALOG_BODY_N.size().y;
        let s = atlas::DIALOG_BODY_S.size().y;
        let w = atlas::DIALOG_BODY_W.size().x;
        let e = atlas::DIALOG_BODY_E.size().x;

        geom.draw_ui(atlas::DIALOG_BODY_NW, lower.inset(0, -w, 0, -n).min);
        geom.draw_ui(atlas::DIALOG_BODY_NE, lower.inset(-e, 0, 0, -n).min);
        geom.draw_ui(atlas::DIALOG_BODY_SW, lower.inset(0, -w, -s, 0).min);
        geom.draw_ui(atlas::DIALOG_BODY_SE, lower.inset(-e, 0, -s, 0).min);

        geom.draw_ui_tiled(atlas::DIALOG_BODY_N, lower.inset(w, e, 0, -n));
        geom.draw_ui_tiled(atlas::DIALOG_BODY_S, lower.inset(w, e, -s, 0));
        geom.draw_ui_tiled(atlas::DIALOG_BODY_W, lower.inset(0, -w, n, s));
        geom.draw_ui_tiled(atlas::DIALOG_BODY_E, lower.inset(-e, 0, n, s));

        geom.draw_ui_tiled(atlas::DIALOG_BODY_CENTER, lower.inset(w, e, n, s));


        let l = atlas::DIALOG_TITLE_LEFT.size().x;
        let r = atlas::DIALOG_TITLE_RIGHT.size().x;

        geom.draw_ui(atlas::DIALOG_TITLE_LEFT,  title.inset(0, -l, 0, 0).min);
        geom.draw_ui(atlas::DIALOG_TITLE_RIGHT, title.inset(-r, 0, 0, 0).min);
        geom.draw_ui_tiled(atlas::DIALOG_TITLE_CENTER, title.inset(l, r, 0, 0));


        self.dyn.with_title(|title_str| {
            let font = &fonts::TITLE;
            let text_width = font.measure_width(title_str);
            let text =
                Region::sized(V2::new(text_width as i32, font.height as i32))
                    .align(title, Align::Center, Align::Center) + V2::new(0, 1);
            geom.draw_str(font, title_str, text.min);
        });
    }
}

// There are 7 pixels on each side of the title bar that are too far up for a spacer to connect to.
// There are 8 pixels on either side of the spacer graphic that are actually transparent, and don't
// count.
const DIALOG_SPACER_INSET: i32 = 7 - 8;
