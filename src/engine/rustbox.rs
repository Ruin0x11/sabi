use std::cell::RefCell;
use std::collections::VecDeque;
use std::error::Error;
use std::default::Default;

use point::Point;
use rustbox::{self, RustBox, Event};
use rustbox::Color as RustboxColor;
use rustbox::Key as RustboxKey;
use slog::Logger;

use color::{Color, Color216, Color16};
use engine::canvas::Canvas_;
use glyph::{self, Glyph};
use keys::{self, Key, KeyCode, NumkeyType};
use log;
use ui::{WindowKind};

const MESSAGE_WINDOW_HEIGHT: i32 = 5;

impl From<rustbox::Key> for Key {
    fn from(rb_key: rustbox::Key) -> Key {
        match rb_key {
            RustboxKey::Char(c) => {
                Key::from(c)
            },
            RustboxKey::Ctrl(c) => {
                let mut key = Key::from(c);
                key.ctrl = true;
                key
            },
            RustboxKey::F(n)    => {
                let keycode = keys::Key::numkey_code_from_digit(n, NumkeyType::Function);
                Key::from(keycode)
            }

            RustboxKey::Tab       => Key::from(KeyCode::Tab),
            RustboxKey::Enter     => Key::from(KeyCode::Enter),
            RustboxKey::Esc       => Key::from(KeyCode::Esc),
            RustboxKey::Backspace => Key::from(KeyCode::Backspace),
            RustboxKey::Right     => Key::from(KeyCode::Right),
            RustboxKey::Left      => Key::from(KeyCode::Left),
            RustboxKey::Up        => Key::from(KeyCode::Up),
            RustboxKey::Down      => Key::from(KeyCode::Down),
            RustboxKey::Delete    => Key::from(KeyCode::Delete),
            RustboxKey::Insert    => Key::from(KeyCode::Insert),

            RustboxKey::Home      => Key::from(KeyCode::Home),
            RustboxKey::End       => Key::from(KeyCode::End),
            RustboxKey::PageUp    => Key::from(KeyCode::PageUp),
            RustboxKey::PageDown  => Key::from(KeyCode::PageDown),
            _                     => Key::from(KeyCode::Unknown(' ')),
        }
    }
}

impl Into<rustbox::Color> for Color16 {
    fn into(self) -> rustbox::Color {
        match self {
            Color16::Black   => rustbox::Color::Black,
            Color16::Red     => rustbox::Color::Red,
            Color16::Green   => rustbox::Color::Green,
            Color16::Yellow  => rustbox::Color::Yellow,
            Color16::Blue    => rustbox::Color::Blue,
            Color16::Magenta => rustbox::Color::Magenta,
            Color16::Cyan    => rustbox::Color::Cyan,
            Color16::White   => rustbox::Color::White,
            _                => rustbox::Color::Default,
        }
    }
}

impl Into<rustbox::Color> for Color216 {
    fn into(self) -> rustbox::Color {
        rustbox::Color::Byte(self.to_u8() as u16)
    }
}

pub struct RustboxCanvas {
    logger: Logger,
    root: RefCell<RustBox>,
    wants_close: bool,
    output_mode: rustbox::OutputMode,
    message_buffer: VecDeque<String>,

    camera_x: i32,
    camera_y: i32,
}

impl RustboxCanvas {
    pub fn new(_display_size: Point,
               _window_title: &str) -> RustboxCanvas {
        let output_mode = rustbox::OutputMode::Normal;

        let root = match RustBox::init(rustbox::InitOptions {
            output_mode: output_mode,
            ..Default::default()
        }) {
            Result::Ok(v) => RefCell::new(v),
            Result::Err(e) => panic!("{}", e),
        };

        let canvas = RustboxCanvas {
            logger: log::make_logger("graphics"),
            root: root,
            wants_close: false,
            output_mode: output_mode,
            message_buffer: VecDeque::new(),
            camera_x: 0,
            camera_y: 0,
        };

        info!(canvas.logger, "Rustbox canvas initialized, output mode: {:?}", output_mode);

        canvas
    }
}

impl Canvas_ for RustboxCanvas {
    fn print_info(&self) {
        println!("Rustbox, for the greater good!");
    }

    fn width(&self) -> i32 {
        self.root.borrow().width() as i32
    }

    fn height(&self) -> i32 {
        self.root.borrow().height() as i32
    }

    fn clear(&self) {
        self.root.borrow_mut().clear();
    }

    fn present(&self) {
        self.root.borrow_mut().present();
    }

    fn set_camera(&mut self, x: i32, y: i32) {
        self.camera_x = x;
        self.camera_y = y;
    }

    fn translate_pos(&self, world_x: i32, world_y: i32) -> (i32, i32) {
        let w = self.width();
        let h = self.height() - MESSAGE_WINDOW_HEIGHT;
        (world_x - self.camera_x + (w / 2), world_y - self.camera_y + (h / 2))
    }

    fn get_input(&self) -> Vec<Key> {
        let mut keys = Vec::new();
        // NOTE: If it gets bad, switch to peek_event
        match self.root.borrow().poll_event(false) {
            Ok(ev) => match ev {
                // NOTE: Due to the way terminals work, Rustbox sends an Esc
                // keypress along with the keycode when using Alt with a key.
                Event::KeyEvent(key) => keys.push(Key::from(key)),
                Event::NoEvent => (),
                _              => (),
            },
            Err(_) => (),
        }
        keys
    }

    fn print_glyph(&self, x: i32, y: i32, glyph: Glyph) {
        let (x, y) = self.translate_pos(x, y);
        let rend_glyph = glyph::lookup_ascii(glyph);
        let color_fg = Color16::from(rend_glyph.color_fg.clone()).into();
        let color_bg = Color16::from(rend_glyph.color_bg.clone()).into();
        self.root.borrow_mut().print_char(x as usize,
                             y as usize,
                             rustbox::RB_NORMAL,
                             color_fg,
                             color_bg,
                             rend_glyph.ch)
    }

    fn close_window(&mut self) {
        self.wants_close = true;
    }

    fn window_closed(&self) -> bool {
        self.wants_close
    }

    fn print_str(&self, x: i32, y: i32, s: &str) {
        self.root.borrow_mut().print(x as usize,
                        y as usize,
                        rustbox::RB_NORMAL,
                        RustboxColor::White,
                        RustboxColor::Default,
                        s);
    }

    fn draw_window(&self, kind: WindowKind) {
        match kind {
            WindowKind::Message => {
                let w = self.width();
                let h = self.height() - 1;
                self.print_str(0, h, "-".repeat(w as usize).as_str());
                let meswin_top = h - MESSAGE_WINDOW_HEIGHT;
                for i in meswin_top..h {
                    self.print_str(0, i, " ".repeat(w as usize).as_str());
                }
            }
        }
    }

    fn print_messages(&self) {
        let w = self.width();
        let h = self.height() - 1;
        for (idx, message) in self.message_buffer.iter().take(MESSAGE_WINDOW_HEIGHT as usize).enumerate() {
            self.print_str(0, h - idx as i32, message);
        }
        self.print_str(0, h - MESSAGE_WINDOW_HEIGHT, "-".repeat(w as usize).as_str());
    }

    fn update_message_buffer(&mut self, messages: Vec<String>) {
        for (idx, message) in messages.into_iter().enumerate() {
            self.message_buffer.push_front(message);
            if idx > 0 && (idx % 5) == 0 {
                self.print_messages();
                self.print_str(0, 0, "--MORE--");
                self.get_input();
            }
        }
        self.print_messages();
    }
}

