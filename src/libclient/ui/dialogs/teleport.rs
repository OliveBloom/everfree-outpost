use std::prelude::v1::*;
use types::*;
use std::cell::Cell;
use common_proto::game::Request;
use outpost_ui::geom::Point;
use outpost_ui::widget::Widget as WidgetTrait;
use physics::v3::{V2, Region, Align};

use client::ClientObj;
use data::{Data, RecipeDef};
use input::EventStatus;
use ui::crafting;
use ui::dialogs;
use ui::geom::Geom;
use ui::input::{KeyAction, ActionEvent};
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
        let mut ctx = ContextImpl::new(self.dyn, geom, rect);

        /*
        TextListItem {
            text: "hello",
            text_width: 100,
        }.on_paint(&mut ctx);
        */

        TextList {
            top: &Cell::new(0),
            focus: &Cell::new(0),
            items: &[
                "good morning".to_owned(),
                "everfree outpost".to_owned(),
                "!!!".to_owned(),
            ],
            size: Point { x: 300, y: 200 },
        }.on_paint(&mut ctx);

        /*
        let l = Label::new("hello, outpost");
        let st = Cell::new(0);
        ScrollPane::new(&st, from_v2(V2::new(300, 200)), l).on_paint(&mut ctx);
        */

    }

    /*
    fn on_key(&mut self, key: ActionEvent) -> EventStatus {
    }

    fn on_mouse_move(&mut self, ctx: &mut Context, rect: Region<V2>) -> EventStatus {
    }

    fn on_mouse_down(&mut self,
                     ctx: &mut Context,
                     rect: Region<V2>,
                     evt: ButtonEvent) -> EventStatus {
    }

    fn on_mouse_up(&mut self,
                   ctx: &mut Context,
                   rect: Region<V2>,
                   evt: ButtonEvent) -> EventStatus {
    }
    */
}
