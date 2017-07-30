use outpost_ui::context;
use outpost_ui::context::{
    Context as ContextTrait,
    ScrollBarStyle as ScrollBarStyleTrait,
};
use outpost_ui::event::{KeyEvent, KeyInterp};
use outpost_ui::geom::{Point, Rect};
use physics::v3::{V2, scalar, Region};

use data::Data;
use fonts::{self, FontMetrics, FontMetricsExt};
use input;
use ui::Context as Context1;
use ui::atlas::{self, AtlasEntry};
use ui::geom::Geom;
use ui::input::{KeyAction, ActionEvent};
use ui2::util::*;

pub struct ContextImpl<'d, 'a> {
    data: &'d Data,
    geom: Option<&'a mut Geom>,
    state: context::CommonState,
}

impl<'d, 'a> ContextImpl<'d, 'a> {
    pub fn new(data: &'d Data,
               bounds: Region<V2>) -> ContextImpl<'d, 'a> {
        ContextImpl {
            data: data,
            geom: None,
            state: context::CommonState::new(from_region2(bounds)),
        }
    }

    pub fn set_geom(&mut self, geom: &'a mut Geom) {
        self.geom = Some(geom);
    }

    pub fn add_mouse_info(&mut self, ctx: &Context1, up: bool) {
        self.state.mouse_pos = Some(from_v2(ctx.mouse_pos));
        // ui::Context clears the mouse_down field before dispatching mouse-up events to widgets,
        // but libui widgets expect mouse_down_pos to still be set during mouse-up handling.  So we
        // pretend mouse_down is still set when handling mouse-up events.
        if ctx.mouse_down || up {
            self.state.mouse_down_pos = Some(from_v2(ctx.mouse_down_pos));
        }
    }

    fn geom(&mut self) -> &mut Geom {
        self.geom.as_mut().expect("ui context has no geom")
    }
}

impl<'d, 'a> context::Context for ContextImpl<'d, 'a> {
    fn state(&self) -> &context::CommonState { &self.state }
    fn state_mut(&mut self) -> &mut context::CommonState { &mut self.state }

    type Key = ActionEvent;
    fn interp_key(&self, evt: KeyEvent<Self>) -> Option<KeyInterp> {
        let interp = match evt {
            KeyEvent::Down(act) => {
                let mag = if act.shift() { 10 } else { 1 };
                match act.code {
                    KeyAction::MoveLeft => KeyInterp::FocusX(mag),
                    KeyAction::MoveRight => KeyInterp::FocusX(-mag),
                    KeyAction::MoveDown => KeyInterp::FocusY(mag),
                    KeyAction::MoveUp => KeyInterp::FocusY(-mag),

                    KeyAction::Select => KeyInterp::Activate,

                    _ => return None,
                }
            },
            _ => return None,
        };
        Some(interp)
    }

    type Button = input::Button;

    type TextStyle = TextStyle;
    fn draw_str(&mut self, s: &str, style: TextStyle) {
        let pos = to_v2(self.cur_bounds().min);
        self.geom().draw_str(style.metrics, s, pos);
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
        let bounds = to_region2(self.cur_bounds()).inset(0, -style.width(), 0, 0);
        let size = bounds.size();

        match style {
            ScrollBarStyle::Default => {
                // Whole bar is the width of the handle.  The caps are 2px narrower than the
                // handle, and the background is 6px narrower.
                let bg_rect = bounds.inset(3, 3, 0, 0);
                self.geom().draw_ui_tiled(atlas::SCROLL_BAR_BAR_BELOW, bg_rect);

                let cap_size = atlas::SCROLL_BAR_CAP.size();
                let cap_rect1 = bounds.inset(1, 1, 0, -cap_size.y);
                self.geom().draw_ui(atlas::SCROLL_BAR_CAP, cap_rect1.min);
                let cap_rect2 = bounds.inset(1, 1, -cap_size.y, 0);
                self.geom().draw_ui(atlas::SCROLL_BAR_CAP, cap_rect2.min);

                // Caps are each 4px high, but can overlap by 1px with the handle.  The handle
                // itself is 5px high.  Valid handle offsets are 0 to max_offset, inclusive.
                let max_offset = size.y - 3 * 2 - 5;
                // Top handle is 4px, but 1px can overlap.
                let base_offset = 3;
                let offset = base_offset + 
                    if max != 0 { max_offset * val as i32 / max as i32 } else { 0 };
                let handle_pos = bounds.min + V2::new(0, offset);
                self.geom().draw_ui(atlas::SCROLL_BAR_THUMB, handle_pos);
            },
        }
    }


    // Custom impl of `with_surface` to set clipping on the `geom` as well.
    fn with_surface<F: FnOnce(&mut Self) -> R, R>(&mut self,
                                                  size: Point,
                                                  src_pos: Point,
                                                  dest_rect: Rect,
                                                  func: F) -> R {
        let old = self.state_mut().push_surface(size, src_pos, dest_rect);
        let clip = self.state().clip.unwrap();
        let geom_old = if let Some(geom) = self.geom.as_mut() {
            geom.push_clip(to_region2(clip))
        } else {
            None
        };

        let r = func(self);

        self.state_mut().pop_surface(old);
        if let Some(geom) = self.geom.as_mut() {
            geom.pop_clip(geom_old);
        }

        r
    }
}


/// This trait is essentially an inherent impl on `ContextImpl`, but it lets us hide the lifetimes
/// from users.  This simplifies a lot of signatures in `ui2::widgets`.
pub trait Context: context::Context<TextStyle=TextStyle,
                                    ButtonStyle=ButtonStyle,
                                    ScrollBarStyle=ScrollBarStyle> {
    fn draw_ui(&mut self, atlas: AtlasEntry, pos: Point);
    fn draw_ui_tiled(&mut self, atlas: AtlasEntry, rect: Rect);
}

impl<'d, 'a> Context for ContextImpl<'d, 'a> {
    fn draw_ui(&mut self, atlas: AtlasEntry, pos: Point) {
        let pos = to_v2(pos + self.cur_bounds().min);
        self.geom().draw_ui(atlas, pos);
    }

    fn draw_ui_tiled(&mut self, atlas: AtlasEntry, rect: Rect) {
        let rect = to_region2(rect + self.cur_bounds().min);
        self.geom().draw_ui_tiled(atlas, rect);
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
