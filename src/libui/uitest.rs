extern crate png_file;
extern crate sdl2;

#[macro_use] extern crate ui;


use std::fs::File;
use std::io::{self, Read, Write};
use std::marker::PhantomData;
use std::path::Path;
use std::sync::mpsc::{Sender, Receiver, channel, TryRecvError};
use std::thread;

use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::{Keycode, Mod};
use sdl2::mouse::MouseButton;
use sdl2::pixels::Color;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;

use ui::context::{Context, CommonState};
use ui::event::{KeyEvent, MouseEvent, UIResult};
use ui::geom;
use ui::widgets;
use ui::widgets::container::{ChildWidget, Align};


mod ctx {
    use std::marker::PhantomData;
    use sdl2::keyboard::Keycode;
    use sdl2::pixels::Color;
    use sdl2::rect::Rect as SdlRect;
    use sdl2::render::{Renderer, Texture};

    use ui::context::{Context, CommonState, ButtonState};
    use ui::context::{TextStyle, ButtonStyle};
    use ui::geom::*;


    trait RenderBits {
        fn open<F: FnOnce(&mut Renderer, &Texture) -> R, R>(&mut self, f: F) -> R;
    }

    pub struct RenderBitsImpl<'w: 'a, 'a> {
        pub renderer: &'a mut Renderer<'w>,
        pub texture: &'a Texture,
    }

    impl<'w, 'a> RenderBits for RenderBitsImpl<'w, 'a> {
        fn open<F: FnOnce(&mut Renderer, &Texture) -> R, R>(&mut self, f: F) -> R {
            f(self.renderer, self.texture)
        }
    }


    pub struct SdlContext<RB> {
        pub state: CommonState,
        pub bits: RB,
    }


    impl<RB: RenderBits> Context for SdlContext<RB> {
        type Key = Keycode;
        type Button = ();

        fn state(&self) -> &CommonState {
            &self.state
        }

        fn state_mut(&mut self) -> &mut CommonState {
            &mut self.state
        }


        type TextStyle = SdlTextStyle;

        fn draw_str(&mut self, s: &str, style: Self::TextStyle) {
            let base = self.cur_bounds().min;
            for (i, c) in s.chars().enumerate() {
                let code = if (c as u32) < 128 { c as u32 as u8 } else { 168 };
                let x = code % 16;
                let y = code / 16;
                let src = SdlRect::new(x as i32 * 16, y as i32 * 16, 16, 16);
                let dst = SdlRect::new(base.x + i as i32 * 16, base.y, 16, 16);
                self.bits.open(|r, t| {
                    r.copy(t, Some(src), Some(dst)).unwrap();
                });
            }
        }


        type ButtonStyle = SdlButtonStyle;

        fn draw_button(&mut self, style: Self::ButtonStyle, state: ButtonState) {
            let bounds = self.cur_bounds();
            self.bits.open(|r, _| {

                let _0 = if style == SdlButtonStyle::Checked { 255 } else { 0 };

                let mut color = match state {
                    ButtonState::Up => Color::RGB(200, 50, _0),
                    ButtonState::Hover => Color::RGB(_0, 50, 200),
                    ButtonState::Active => Color::RGB(250, 250, _0),
                    ButtonState::Down => Color::RGB(50, 180, _0),
                };

                r.set_draw_color(color);
                r.fill_rect(super::sdl_rect(bounds));
            });
        }
    }


    #[derive(Clone, Copy, Debug)]
    pub enum SdlTextStyle {
        Normal,
    }

    impl Default for SdlTextStyle {
        fn default() -> SdlTextStyle {
            SdlTextStyle::Normal
        }
    }

    impl TextStyle for SdlTextStyle {
        fn text_size(&self, s: &str) -> Point {
            Point {
                x: 16 * s.len() as i32,
                y: 16,
            }
        }
    }


    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub enum SdlButtonStyle {
        Normal,
        Checked,
    }

    impl Default for SdlButtonStyle {
        fn default() -> SdlButtonStyle {
            SdlButtonStyle::Normal
        }
    }

    impl ButtonStyle for SdlButtonStyle {
        fn border_size(&self) -> (Point, Point) {
            (Point { x: 5, y: 3 },
             Point { x: 5, y: 3 })
        }

        fn default_off() -> Self {
            SdlButtonStyle::Normal
        }

        fn default_on() -> Self {
            SdlButtonStyle::Checked
        }
    }
}

pub fn sdl_rect(r: geom::Rect) -> Rect {
    Rect::new(r.min.x,
              r.min.y,
              (r.max.x - r.min.x) as u32,
              (r.max.y - r.min.y) as u32)
}

pub fn main() {
    // SDL init
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("UI Test", 800, 600)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.renderer().present_vsync().build().unwrap();

    let (font_info, font_buf) = png_file::read_png_file(
        Path::new("junk/Raving_1280x400.png")).unwrap();
    let mut font_tex = canvas.create_texture_static(PixelFormatEnum::RGBX8888, 
                                                    font_info.width as u32,
                                                    font_info.height as u32).unwrap();
    font_tex.update(None, &font_buf, font_info.width as usize * 4);

    let bits = ctx::RenderBitsImpl {
        renderer: &mut canvas,
        texture: &font_tex,
    };
    let mut ctx = ctx::SdlContext {
        state: CommonState::new(geom::Rect::new(0, 0, 800, 600)),
        bits: bits,
    };
    let root_rect = geom::Rect::new(0, 0, 400, 150);
    //let mut root = widgets::button::Button::new("hello, world");
    //let mut root = widgets::button::CheckBox::new("hello, world", false);
    let mut root = widgets::container::Box::horiz(contents![
        ChildWidget::new(widgets::text::Label::new("hello, world"), |_| ()),
        ChildWidget::new(widgets::button::Button::new("hello, world"), |_| ()).align(Align::Center),
        ChildWidget::new(widgets::text::Label::new("poni poni"), |_| ()).align(Align::End),
        ChildWidget::new(widgets::button::CheckBox::new("hello, world", false), |_| ()),
        ChildWidget::new(widgets::text::Label::new("aeiou"), |_| ()),
    ]).spacing(5);

    // Main loop

    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut visible = true;

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },

                Event::KeyDown { keycode: Some(code), keymod, .. } => {
                    let evt = ctx.dispatch_key(KeyEvent::Down(code), root_rect, &mut root);
                    println!("keydown {:?} -> {:?}", code, evt)
                },
                Event::KeyUp { keycode: Some(code), keymod, .. } => {
                    let evt = ctx.dispatch_key(KeyEvent::Up(code), root_rect, &mut root);
                    println!("keyup {:?} -> {:?}", code, evt)
                },

                Event::MouseMotion { x, y, .. } => {
                    ctx.state.record_mouse_move(geom::Point { x: x, y: y });
                    let evt = ctx.dispatch_mouse(MouseEvent::Move, root_rect, &mut root);
                    println!("mousemove {},{} -> {:?}", x, y, evt);
                },
                Event::MouseButtonDown { mouse_btn, x, y, .. } => {
                    if mouse_btn == MouseButton::Left {
                        ctx.state.record_mouse_down(geom::Point { x: x, y: y });
                    }
                    let evt = ctx.dispatch_mouse(MouseEvent::Down(()), root_rect, &mut root);
                    println!("mousedown {:?} {},{} -> {:?}", mouse_btn, x, y, evt);
                },
                Event::MouseButtonUp { mouse_btn, x, y, .. } => {
                    let evt = ctx.dispatch_mouse(MouseEvent::Up(()), root_rect, &mut root);
                    if mouse_btn == MouseButton::Left {
                        ctx.state.record_mouse_up(geom::Point { x: x, y: y });
                    }
                    println!("mouseup {:?} {},{} -> {:?}", mouse_btn, x, y, evt);

                    //if let UIResult::Event(b) = evt {
                        //root = root.checked(b);
                    //}
                },


                Event::Window { win_event, .. } => {
                    match win_event {
                        WindowEvent::Hidden => {
                            visible = false;
                        },
                        WindowEvent::Shown => {
                            visible = true;
                        },
                        _ => {},
                    }
                }
                _ => {}
            }
        }

        if visible {
            {
                let canvas = &mut *ctx.bits.renderer;
                canvas.set_draw_color(Color::RGB(0, 0, 0));
                canvas.clear();
                canvas.set_draw_color(Color::RGB(255, 210, 0));
                canvas.fill_rect(Rect::new(10, 10, 780, 580));
            }

            ctx.dispatch_paint(root_rect, &mut root);

            ctx.bits.renderer.present();
        }
    }
}

