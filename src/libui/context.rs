use std::mem;

use event::{KeyEvent, MouseEvent, UIResult};
use geom::{Point, Rect};
use widget::Widget;


pub enum Void {}

pub struct CommonState {
    pub bounds: Rect,
    pub mouse_pos: Option<Point>,
    pub mouse_down_pos: Option<Point>,
}

impl CommonState {
    pub fn new(bounds: Rect) -> CommonState {
        CommonState {
            bounds: bounds,
            mouse_pos: None,
            mouse_down_pos: None,
        }
    }

    pub fn record_mouse_move(&mut self, pos: Point) {
        self.mouse_pos = Some(pos);
    }

    pub fn record_mouse_down(&mut self, pos: Point) {
        self.mouse_pos = Some(pos);
        self.mouse_down_pos = Some(pos);
    }

    pub fn record_mouse_up(&mut self, pos: Point) {
        self.mouse_pos = Some(pos);
        self.mouse_down_pos = None;
    }
}

pub trait Context: Sized {
    type Key: Clone;
    type Button: Clone;

    fn state(&self) -> &CommonState;
    fn state_mut(&mut self) -> &mut CommonState;

    type TextStyle: TextStyle;
    fn draw_str(&mut self, s: &str, style: Self::TextStyle);

    type ButtonStyle: ButtonStyle;
    fn draw_button(&mut self, style: Self::ButtonStyle, state: ButtonState);


    fn cur_bounds(&self) -> Rect {
        self.state().bounds
    }

    fn with_bounds<F: FnOnce(&mut Self) -> R, R>(&mut self, bounds: Rect, func: F) -> R {
        // TODO: clip better
        let base = self.state().bounds.min;
        let abs_bounds = Rect {
            min: bounds.min + base,
            max: bounds.max + base,
        };

        let old_bounds = mem::replace(&mut self.state_mut().bounds, abs_bounds);
        let r = func(self);
        self.state_mut().bounds = old_bounds;
        r
    }

    /// Is the mouse inside the current bounds?
    fn mouse_over(&self) -> bool {
        let s = self.state();
        s.mouse_pos.map_or(false, |pos| s.bounds.contains(pos))
    }

    /// Was the mouse pressed while inside the current bounds?
    fn mouse_pressed_over(&self) -> bool {
        let s = self.state();
        s.mouse_down_pos.map_or(false, |pos| s.bounds.contains(pos))
    }


    fn dispatch_paint<W: Widget<Self>>(&mut self,
                                       rect: Rect,
                                       w: &mut W) {
        self.with_bounds(rect, |ctx| {
            w.on_paint(ctx);
        });
    }

    fn dispatch_key<W: Widget<Self>>(&mut self,
                                     evt: KeyEvent<Self>,
                                     rect: Rect,
                                     w: &mut W) -> UIResult<W::Event> {
        self.with_bounds(rect, |ctx| {
            w.on_key(ctx, evt)
        })
    }

    fn dispatch_mouse<W: Widget<Self>>(&mut self,
                                       evt: MouseEvent<Self>,
                                       rect: Rect,
                                       w: &mut W) -> UIResult<W::Event> {
        self.with_bounds(rect, |ctx| {
            w.on_mouse(ctx, evt)
        })
    }
}

pub enum ButtonState {
    Up,
    Down,
    Hover,
    Active,
}


pub trait TextStyle: Sized+Copy+Default {
    fn text_size(&self, s: &str) -> Point;
}

pub trait ButtonStyle: Sized+Copy+Default {
    fn border_size(&self) -> (Point, Point);

    fn default_off() -> Self;
    fn default_on() -> Self;
}
