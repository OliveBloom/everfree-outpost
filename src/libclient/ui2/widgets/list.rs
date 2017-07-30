use std::prelude::v1::*;
use std::cell::{Cell, RefCell};
use std::cmp;

use outpost_ui::context::Context as ContextTrait;
use outpost_ui::context::Focus;
use outpost_ui::event::{KeyEvent, MouseEvent, UIResult};
use outpost_ui::geom::{Point, Rect};
use outpost_ui::geom::Vertical;
use outpost_ui::widget::Widget;
use outpost_ui::widgets::container::{Group, ChildWidget, GenWidgets, Contents};
use outpost_ui::widgets::scroll::ScrollPane;
use outpost_ui::widgets::text::WrappedLabel;
use physics::v3::{V2, Vn, scalar, Region};

use ui::atlas;
use ui2::context::Context;
use ui2::util::*;


pub struct TextListItem<'a> {
    pub text: &'a str,
    pub text_width: i32,
}

impl<'a> TextListItem<'a> {
    fn inner<Ctx: Context+'a>(&self) -> impl Widget<Ctx>+'a {
        WrappedLabel::new(self.text, self.text_width - 2 - 2)
    }
}

impl<'a, Ctx: Context> Widget<Ctx> for TextListItem<'a> {
    type Event = ();

    fn min_size(&self) -> Point {
        self.inner::<Ctx>().min_size() + Point { x: 9 + 2, y: 2 + 2 }
    }

    fn on_paint(&self, ctx: &mut Ctx) {
        let size = ctx.cur_bounds().size();
        // Background bar currently has a fixed size, 13px high.
        let size = Point { x: size.x, y: cmp::min(size.y, 13) };

        let fill_rect = Rect::sized(size).inset(2, 0, 1, 0);
        ctx.draw_ui(atlas::SCROLL_LIST_ENTRY_LEFT, Point { x: 0, y: 0 });
        ctx.draw_ui(atlas::SCROLL_LIST_ENTRY_RIGHT, Point { x: size.x - 1, y: 0 });
        ctx.draw_ui_tiled(atlas::SCROLL_LIST_ENTRY_MID, fill_rect);

        if ctx.state().focus >= Focus::Semiactive {
            ctx.draw_ui(atlas::SCROLL_LIST_MARKER, Point { x: 2, y: 2 });
        }

        let text_bounds = Rect::sized(size).inset(9, 2, 2, 2);
        ctx.with_bounds(text_bounds, |ctx| {
            self.inner().on_paint(ctx);
        });
    }

    fn on_key(&self, ctx: &mut Ctx, evt: KeyEvent<Ctx>) -> UIResult<Self::Event> {
        // TODO
        UIResult::Unhandled
    }

    fn on_mouse(&self, ctx: &mut Ctx, evt: MouseEvent<Ctx>) -> UIResult<Self::Event> {
        match evt {
            MouseEvent::Down(_) => UIResult::NoEvent,
            MouseEvent::Up(_) => {
                if ctx.mouse_pressed_over() {
                    UIResult::Event(())
                } else {
                    UIResult::NoEvent
                }
            },
            _ => UIResult::Unhandled,
        }
    }
}


pub struct TextList<'s, 'a> {
    pub top: &'s Cell<i32>,
    pub focus: &'s Cell<usize>,
    pub items: &'a [String],
    pub size: Point,
}

impl<'s, 'a> TextList<'s, 'a> {
    fn inner<Ctx: Context+'a>(&self) ->
            ScrollPane<'s, Ctx,
                Group<'s, Ctx, Vertical, usize,
                    impl Contents<Ctx, usize>+'a>> {
        let items = self.items;
        let width = self.size.x;
        let contents = GenWidgets::new(0 .. items.len(), move |idx| {
            ChildWidget::new(TextListItem {
                text: &items[idx],
                text_width: width,
            }, move |()| idx)
        });
        ScrollPane::new(self.top, self.size, Group::vert(self.focus, contents))
    }
}

impl<'s, 'a, Ctx: Context> Widget<Ctx> for TextList<'s, 'a> {
    type Event = usize;

    fn min_size(&self) -> Point {
        self.size
    }

    fn on_paint(&self, ctx: &mut Ctx) {
        self.inner().on_paint(ctx);
    }

    fn on_key(&self, ctx: &mut Ctx, evt: KeyEvent<Ctx>) -> UIResult<Self::Event> {
        self.inner().on_key(ctx, evt)
    }

    fn on_mouse(&self, ctx: &mut Ctx, evt: MouseEvent<Ctx>) -> UIResult<Self::Event> {
        self.inner().on_mouse(ctx, evt)
    }
}
