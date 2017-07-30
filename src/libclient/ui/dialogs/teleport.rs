use std::prelude::v1::*;
use types::*;
use std::cell::Cell;
use common_proto::extra_arg::{self, ExtraArg, SimpleArg};
use common_proto::game::Request;
use outpost_ui::event::{KeyEvent, MouseEvent, UIResult};
use outpost_ui::geom::Point;
use outpost_ui::widget::Widget as WidgetTrait;
use physics::v3::{V2, Region, Align, scalar};

use client::ClientObj;
use data::{Data, RecipeDef};
use input::EventStatus;
use input::{Button, ButtonEvent};
use ui::Context as Context1;
use ui::crafting;
use ui::dialogs;
use ui::input::ActionEvent;
use ui::geom::Geom;
use ui::inventory;
use ui::scroll_list;
use ui::util;
use ui::widget::*;
use ui2::context::ContextImpl;
use ui2::util::*;
use ui2::widgets::list::TextList;
use util::hash;


pub struct Teleport {
    dest_names: Vec<String>,
    top: Cell<i32>,
    focus: Cell<usize>,
    /// Stupid hack to deal with the fact that `on_key` doesn't get the current `rect`.
    last_rect: Region<V2>,
}

impl Teleport {
    pub fn new(dest_names: Vec<String>) -> Teleport {
        Teleport {
            dest_names: dest_names,
            top: Cell::new(0),
            focus: Cell::new(0),
            last_rect: Region::sized(V2::new(0, 0)),
        }
    }

    fn inner(&self) -> TextList {
        TextList {
            top: &self.top,
            focus: &self.focus,
            items: &self.dest_names,
            size: Point { x: 200, y: 250 },
        }
    }

    fn handle_event(&self, idx: usize) -> EventStatus {
        let dest = self.dest_names[idx].clone();
        EventStatus::Action(box move |c: &mut ClientObj| {
            let mut m = extra_arg::Map::new();
            m.insert(SimpleArg::Str("dest".to_owned()),
                     ExtraArg::Str(dest));
            let e = ExtraArg::Map(m);

            let msg = Request::InteractWithArgs(c.msg_time(), e);
            c.platform().send_message(msg);

            c.ui().root.dialog.inner = dialogs::AnyDialog::none();
        })
    }
}

impl<'a> Widget for WidgetPack<'a, Teleport, Data> {
    fn size(&mut self) -> V2 {
        let size = <TextList as WidgetTrait<ContextImpl>>::min_size(&self.state.inner());
        to_v2(size)
    }

    fn walk_layout<V: Visitor>(&mut self, v: &mut V, pos: V2) {}

    fn render(&mut self, geom: &mut Geom, rect: Region<V2>) {
        let mut ctx = ContextImpl::new(self.dyn, rect);
        ctx.set_geom(geom);

        self.state.inner().on_paint(&mut ctx);

        self.state.last_rect = rect;
    }

    fn on_key(&mut self, key: ActionEvent) -> EventStatus {
        let mut ctx = ContextImpl::new(self.dyn, self.state.last_rect);

        let r = self.state.inner().on_key(&mut ctx, KeyEvent::Down(key.code));
        // TODO: keyup?

        match r {
            UIResult::Unhandled => EventStatus::Unhandled,
            UIResult::NoEvent => EventStatus::Handled,
            UIResult::Event(idx) => self.state.handle_event(idx),
        }
    }

    fn on_mouse_down(&mut self,
                     ctx1: &mut Context1,
                     rect: Region<V2>,
                     evt: ButtonEvent) -> EventStatus {
        let mut ctx = ContextImpl::new(self.dyn, rect);
        ctx.add_mouse_info(ctx1, false);

        let evt = match evt.button {
            Button::Left | Button::Middle | Button::Right => MouseEvent::Down(evt.button),
            Button::WheelUp => MouseEvent::Wheel(1),
            Button::WheelDown => MouseEvent::Wheel(-1),
        };
        let r = self.state.inner().on_mouse(&mut ctx, evt);

        match r {
            UIResult::Unhandled => EventStatus::Unhandled,
            UIResult::NoEvent => EventStatus::Handled,
            UIResult::Event(idx) => self.state.handle_event(idx),
        }
    }

    fn on_mouse_up(&mut self,
                   ctx1: &mut Context1,
                   rect: Region<V2>,
                   evt: ButtonEvent) -> EventStatus {
        let mut ctx = ContextImpl::new(self.dyn, rect);
        ctx.add_mouse_info(ctx1, true);

        let evt = match evt.button {
            Button::Left | Button::Middle | Button::Right => MouseEvent::Up(evt.button),
            Button::WheelUp | Button::WheelDown => return EventStatus::Handled,
        };
        let r = self.state.inner().on_mouse(&mut ctx, evt);

        match r {
            UIResult::Unhandled => EventStatus::Unhandled,
            UIResult::NoEvent => EventStatus::Handled,
            UIResult::Event(idx) => self.state.handle_event(idx),
        }
    }
}
