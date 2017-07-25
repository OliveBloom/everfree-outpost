use std::mem;

use event::{KeyEvent, MouseEvent, UIResult};
use geom::{Point, Rect};
use widget::Widget;


pub enum Void {}

pub struct CommonState {
    pub bounds: Rect,
    pub clip: Option<Rect>,
    pub mouse_pos: Option<Point>,
    pub mouse_down_pos: Option<Point>,
}

impl CommonState {
    pub fn new(bounds: Rect) -> CommonState {
        CommonState {
            bounds: bounds,
            clip: None,
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


    /// Restrict the output bounds to the given sub-region of the current output bounds.  Returns a
    /// state value that can be passed to `pop_bounds` to restore the previous bounds.
    fn push_bounds(&mut self, bounds: Rect) -> Rect {
        let abs_bounds = bounds + self.bounds.min;
        mem::replace(&mut self.bounds, abs_bounds)
    }

    fn pop_bounds(&mut self, old: Rect) {
        self.bounds = old;
    }


    fn push_surface(&mut self, size: Point, src_pos: Point, dest_rect: Rect)
                    -> (Rect, Option<Rect>) {
        let new_clip = dest_rect + self.bounds.min;
        // Which point on the current surface coincides with 0,0 on the new surface?
        let new_origin = dest_rect.min - src_pos;
        let new_bounds = Rect::sized(size) + new_origin;
        (mem::replace(&mut self.bounds, new_bounds),
         mem::replace(&mut self.clip, Some(new_clip)))
    }

    fn pop_surface(&mut self, old: (Rect, Option<Rect>)) {
        let (old_bounds, old_clip) = old;
        self.bounds = old_bounds;
        self.clip = old_clip;
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
        let old = self.state_mut().push_bounds(bounds);
        let r = func(self);
        self.state_mut().pop_bounds(old);
        r
    }

    /// Enter a new drawing surface of the given `size`.  The region of the new surface beginning
    /// at `src_pos` will be output to `dest_rect` in the current surface.
    ///
    /// This method is useful for implementing scrollable content.
    fn with_surface<F: FnOnce(&mut Self) -> R, R>(&mut self,
                                                  size: Point,
                                                  src_pos: Point,
                                                  dest_rect: Rect,
                                                  func: F) -> R {
        let old = self.state_mut().push_surface(size, src_pos, dest_rect);
        let r = func(self);
        self.state_mut().pop_surface(old);
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
