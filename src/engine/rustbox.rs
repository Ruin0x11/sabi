use std::error::Error;
use std::default::Default;

use point::Point;
use rustbox::{self, RustBox, Event};
use rustbox::Color as RustboxColor;
use rustbox::Key as RustboxKey;
use slog::Logger;

use color::{Color, Color216, Color16};
use engine::Canvas_;
use glyph::{self, Glyph};
use keys::{self, Key, KeyCode, NumkeyType};
use log;

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
    root: RustBox,
    wants_close: bool,
    output_mode: rustbox::OutputMode,
}

impl RustboxCanvas {
    pub fn new(_display_size: Point,
               _window_title: &str) -> RustboxCanvas {
        let output_mode = rustbox::OutputMode::Normal;

        let root = match RustBox::init(rustbox::InitOptions {
            output_mode: output_mode,
            ..Default::default()
        }) {
            Result::Ok(v) => v,
            Result::Err(e) => panic!("{}", e),
        };

        let canvas = RustboxCanvas {
            logger: log::make_logger("graphics").unwrap(),
            root: root,
            wants_close: false,
            output_mode: output_mode,
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
        self.root.width() as i32
    }

    fn height(&self) -> i32 {
        self.root.height() as i32
    }

    fn clear(&mut self) {
        self.root.clear();
    }

    fn present(&mut self) {
        self.root.present();
    }

    fn get_input(&self) -> Vec<Key> {
        let mut keys = Vec::new();
        // NOTE: If it gets bad, switch to peek_event
        match self.root.poll_event(false) {
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

    fn print_glyph(&mut self, x: i32, y: i32, glyph: Glyph) {
        let rend_glyph = glyph::lookup_ascii(glyph);
        let color_fg = Color16::from(rend_glyph.color_fg.clone()).into();
        let color_bg = Color16::from(rend_glyph.color_bg.clone()).into();
        self.root.print_char(x as usize,
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

    fn print_str(&mut self, x: i32, y: i32, s: &str) {
        self.root.print(x as usize,
                        y as usize,
                        rustbox::RB_NORMAL,
                        RustboxColor::White,
                        RustboxColor::Black,
                        s);
    }

    fn print_message(&mut self, message: &str) {
        let h = self.height() - 1;
        self.print_str(0, h, message);
    }

    fn show_messages(&mut self, messages: Vec<String>) {
        for (i, mes) in messages.iter().enumerate() {
            self.print_message(&mes);
            if i != messages.len() - 1 {
                self.present();
                self.get_input();
                let w = self.width();
                let h = self.height() - 1;
                self.print_str(0, h, " ".repeat(w as usize).as_str());
            }
        }
    }
}

