use std::prelude::v1::*;
use types::*;
use std::cell::Cell;
use common_proto::game::Request;
use outpost_ui::event::{KeyEvent, MouseEvent, UIResult};
use outpost_ui::geom::Point;
use outpost_ui::widget::Widget as WidgetTrait;
use physics::v3::{V2, Region, Align};

use client::ClientObj;
use data::{Data, RecipeDef};
use input::EventStatus;
use input::{Button, ButtonEvent};
use ui::Context as Context1;
use ui::crafting;
use ui::dialogs;
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
}

impl Teleport {
    pub fn new(dest_names: Vec<String>) -> Teleport {
        Teleport {
            dest_names: dest_names,
            top: Cell::new(0),
            focus: Cell::new(0),
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
    }

    /*
    fn on_key(&mut self, key: ActionEvent) -> EventStatus {
    }

    fn on_mouse_move(&mut self, ctx: &mut Context, rect: Region<V2>) -> EventStatus {
    }
    */

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
            UIResult::Event(idx) => {
                info!("mouse event: {}", idx);
                EventStatus::Handled
            },
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
            UIResult::Event(idx) => {
                info!("mouse event: {}", idx);
                EventStatus::Handled
            },
        }
    }
}
