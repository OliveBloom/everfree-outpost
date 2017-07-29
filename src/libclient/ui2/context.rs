use outpost_ui::context;
use outpost_ui::geom::{Point, Rect};
use outpost_ui::context::Context as ContextTrait;
use physics::v3::{V2, scalar, Region};

use data::Data;
use fonts::{self, FontMetrics, FontMetricsExt};
use input;
use ui::atlas;
use ui::geom::Geom;
use ui2::util::*;

pub struct Context<'d, 'a> {
    data: &'d Data,
    geom: &'a mut Geom,
    state: context::CommonState,
}

impl<'d, 'a> Context<'d, 'a> {
    pub fn new(data: &'d Data,
               geom: &'a mut Geom,
               bounds: Region<V2>) -> Context<'d, 'a> {
        Context {
            data: data,
            geom: geom,
            state: context::CommonState::new(from_region2(bounds)),
        }
    }
}

impl<'d, 'a> context::Context for Context<'d, 'a> {
    type Key = input::Key;
    type Button = input::Button;

    fn state(&self) -> &context::CommonState { &self.state }
    fn state_mut(&mut self) -> &mut context::CommonState { &mut self.state }

    type TextStyle = TextStyle;
    fn draw_str(&mut self, s: &str, style: TextStyle) {
        let pos = to_v2(self.cur_bounds().min);
        self.geom.draw_str(style.metrics, s, pos);
    }

    type ButtonStyle = ButtonStyle;
    fn draw_button(&mut self, style: ButtonStyle, state: context::ButtonState) {
        match style {}
    }

    type ScrollBarStyle = ScrollBarStyle;
    fn draw_scroll_bar(&mut self,
                       style: ScrollBarStyle,
                       val: i16,
                       max: i16,
                       top_pressed: bool,
                       bottom_pressed: bool) {
        let bounds = self.cur_bounds();
        match style {
            ScrollBarStyle::Default => {
                unimplemented!()
            },
        }
    }
}


#[derive(Clone, Copy)]
pub struct TextStyle {
    metrics: &'static FontMetrics,
}

impl TextStyle {
    fn new(metrics: &'static FontMetrics) -> TextStyle {
        TextStyle {
            metrics: metrics,
        }
    }
}

impl Default for TextStyle {
    fn default() -> TextStyle {
        TextStyle::new(&fonts::NAME)
    }
}

impl context::TextStyle for TextStyle {
    fn text_size(&self, s: &str) -> Point {
        let width = self.metrics.measure_width(s) as i32;
        Point { x: width, y: self.line_height() }
    }

    fn space_width(&self) -> i32 {
        self.metrics.space_width as i32
    }

    fn line_height(&self) -> i32 {
        self.metrics.height as i32
    }
}


#[derive(Clone, Copy, Debug)]
pub enum ButtonStyle {
}

impl Default for ButtonStyle {
    fn default() -> ButtonStyle {
        unimplemented!()
    }
}

impl context::ButtonStyle for ButtonStyle {
    fn border_size(&self) -> (Point, Point) {
        match *self {}
    }

    fn default_off() -> ButtonStyle {
        unimplemented!()
    }

    fn default_on() -> ButtonStyle {
        unimplemented!()
    }
}


#[derive(Clone, Copy, Debug)]
pub enum ScrollBarStyle {
    Default,
}

impl Default for ScrollBarStyle {
    fn default() -> ScrollBarStyle {
        ScrollBarStyle::Default
    }
}

impl context::ScrollBarStyle for ScrollBarStyle {
    fn width(&self) -> i32 {
        atlas::SCROLL_BAR_THUMB.size().x
    }

    fn handle_height(&self) -> i32 {
        atlas::SCROLL_BAR_THUMB.size().y
    }

    fn top_button_height(&self) -> i32 {
        0
    }

    fn bottom_button_height(&self) -> i32 {
        0
    }
}
