use std::prelude::v1::*;
use std::borrow::Borrow;

use physics::v3::{V2, scalar, Region};

use fonts::{self, FontMetricsExt};
use ui::atlas;
use ui::geom::Geom;
use ui::widget::*;


#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Attr {
    Normal,
    Bold,
}

#[derive(Clone, Debug)]
pub enum Segment<S: Borrow<str>> {
    LineBreak,
    ParaBreak,
    Text(Attr, S),
}

pub fn render<S>(geom: &mut Geom, text: &[Segment<S>], bounds: Region<V2>)
        where S: Borrow<str> {
    let mut x = bounds.min.x;
    let mut y = bounds.min.y;
    let line_height = fonts::NAME.height as i32 + 1;

    for seg in text {
        match *seg {
            Segment::LineBreak => {
                x = bounds.min.x;
                y += line_height;
            },
            Segment::ParaBreak => {
                x = bounds.min.x;
                y += line_height + 2;
            },
            Segment::Text(attr, ref s) => {
                let s = s.borrow();
                let font = match attr {
                    Attr::Normal => &fonts::NAME,
                    Attr::Bold => &fonts::BOLD,
                };
                for word in s.split_whitespace() {
                    let width = font.measure_width(word) as i32;
                    if x + width > bounds.max.x {
                        x = bounds.min.x;
                        y += line_height;
                    }
                    geom.draw_str(font, word, V2::new(x, y));
                    x += width + font.space_width as i32;
                }
            },
        }
    }
}


#[derive(Clone, Copy)]
pub struct TextDisplay {
    pub size: V2,
}

impl TextDisplay {
    pub fn new() -> TextDisplay {
        TextDisplay {
            size: scalar(0),
        }
    }
}

pub struct TextDisplayDyn<'a, S: Borrow<str>+'a> {
    segments: &'a [Segment<S>],
}

impl<'a, S: Borrow<str>> TextDisplayDyn<'a, S> {
    pub fn new(segments: &'a [Segment<S>]) -> TextDisplayDyn<'a, S> {
        TextDisplayDyn {
            segments: segments,
        }
    }
}

impl<'a, 'b, S: Borrow<str>> Widget for WidgetPack<'a, TextDisplay, TextDisplayDyn<'b, S>> {
    fn size(&mut self) -> V2 {
        self.state.size
    }

    fn walk_layout<V: Visitor>(&mut self, _v: &mut V, _pos: V2) {}

    fn render(&mut self, geom: &mut Geom, rect: Region<V2>) {
        let rect = rect.inset(1, 1, 1, 1);
        geom.draw_ui(atlas::INSET_NW_ROUNDED, rect.inset(0, -2, 0, -2).min);
        geom.draw_ui(atlas::INSET_SE_ROUNDED, rect.inset(-2, 0, -2, 0).min);
        geom.draw_ui_tiled(atlas::INSET_N, rect.inset(2, 1, 0, -1));
        geom.draw_ui_tiled(atlas::INSET_S, rect.inset(1, 2, -1, 0));
        geom.draw_ui_tiled(atlas::INSET_W, rect.inset(0, -1, 2, 1));
        geom.draw_ui_tiled(atlas::INSET_E, rect.inset(-1, 0, 1, 2));

        render(geom, self.dyn.segments, rect.inset(2, 2, 2, 2));
    }
}
