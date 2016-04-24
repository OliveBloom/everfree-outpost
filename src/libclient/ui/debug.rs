use std::prelude::v1::*;

use physics::v3::{V2, scalar, Region, Align};

use client::ClientObj;
use debug;
use fonts::{self, FontMetricsExt};
use platform::{Config, ConfigKey};
use ui::{Context, DragData};
use ui::atlas;
use ui::geom::{Geom, Special};
use ui::input::{EventStatus, KeyAction};
use ui::item;
use ui::widget::*;


#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Mode {
    Nothing = 0,
    Framerate = 1,
    Full = 2,
}

pub struct Debug {
    mode: Mode,
}

impl Debug {
    pub fn new() -> Debug {
        Debug {
            mode: Mode::Nothing,
        }
    }

    pub fn init<C: Config>(&mut self, cfg: &C) {
        match cfg.get_int(ConfigKey::DebugShowPanel) {
            1 => self.mode = Mode::Framerate,
            2 => self.mode = Mode::Full,
            _ => self.mode = Mode::Nothing,
        }
    }
}

const FPS_WIDTH: i32 = 64;

const LABEL_WIDTH: i32 = 32;
const ROW_WIDTH: i32 = 128;
const CONTENT_WIDTH: i32 = ROW_WIDTH - LABEL_WIDTH;
const ROW_HEIGHT: i32 = 10;
const GRAPH_HEIGHT: i32 = 20;

impl<'a> Widget for WidgetPack<'a, Debug, &'a debug::Debug> {
    fn size(&mut self) -> V2 {
        match self.state.mode {
            Mode::Nothing => scalar(0),
            Mode::Framerate => V2::new(FPS_WIDTH, ROW_HEIGHT),
            Mode::Full => V2::new(ROW_WIDTH, ROW_HEIGHT * 4 + GRAPH_HEIGHT),
        }
    }

    fn walk_layout<V: Visitor>(&mut self, v: &mut V, pos: V2) {
        // No children
    }

    fn render(&mut self, geom: &mut Geom, rect: Region<V2>) {
        match self.state.mode {
            Mode::Nothing => {},

            Mode::Framerate => {
                let rate = calc_framerate(self.dyn.total_interval);
                let s = format!("{}.{} FPS", rate / 10, rate % 10);
                let width = fonts::NAME.measure_width(&s);
                let offset = rect.size().x - width as i32;
                geom.draw_str(&fonts::NAME, &s, rect.min + V2::new(offset, 0));
            },

            Mode::Full => {
                let step = V2::new(0, ROW_HEIGHT);
                let offset = V2::new(LABEL_WIDTH, 0);

                {
                    let mut row = |idx, label: &str, value: &str| {
                        let pos = rect.min + step * scalar(idx);
                        geom.draw_str(&fonts::NAME, label, pos);
                        geom.draw_str(&fonts::NAME, value, pos + offset);
                    };

                    let rate = calc_framerate(self.dyn.total_interval);
                    row(0, "FPS", &format!("{}.{}", rate / 10, rate % 10));
                    row(1, "Ping", &format!("{} ms", self.dyn.ping));
                    row(2, "Pos", &format!("{:?}", self.dyn.pos));
                    row(3, "Time", &format!("{}.{:03} ({})",
                                            self.dyn.day_time / 1000,
                                            self.dyn.day_time % 1000,
                                            self.dyn.day_phase));
                }

                let graph_pos = rect.min + step * scalar(4);
                let graph_rect = Region::sized(V2::new(ROW_WIDTH, GRAPH_HEIGHT)) + graph_pos;
                let idx = self.dyn.cur_frame;
                let last = (idx + debug::NUM_FRAMES - 1) % debug::NUM_FRAMES;
                geom.special(Special::DebugFrameGraph {
                    rect: graph_rect,
                    cur: idx as u8,
                    last: last as u8,
                    last_time: self.dyn.frame_times[last],
                    last_interval: self.dyn.frame_intervals[last],
                });
            },
        }
    }

    fn on_key(&mut self, key: KeyAction) -> EventStatus {
        if key == KeyAction::ToggleDebugPanel {
            self.state.mode = match self.state.mode {
                Mode::Nothing => Mode::Framerate,
                Mode::Framerate => Mode::Full,
                Mode::Full => Mode::Nothing,
            };

            let new_val = self.state.mode as i32;

            return EventStatus::Action(box move |c: &mut ClientObj| {
                c.platform().config_mut().set_int(ConfigKey::DebugShowPanel, new_val);
            });
        }

        EventStatus::Unhandled
    }
}

/// Calculate the framerate in 1/10s of FPS, based on `Debug::total_interval`.
fn calc_framerate(total_interval: usize) -> usize {
    if total_interval == 0 {
        return 0;
    }

    // total_interval is in milliseconds.  rate is in 1/10s of FPS.
    debug::NUM_FRAMES * 10 * 1000 / total_interval
}
