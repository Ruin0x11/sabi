use glyph::{RenderableGlyph, Glyph};

use std::error::Error;
use std::default::Default;

use rustbox::{self, Color, RustBox, Event};

use euclid::point::Point2D as Point;
use rustbox::Key as RustboxKey;

use engine::Canvas_;
use keys::{self, Key, KeyCode, NumkeyType};

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
                let keycode = keys::numkey_code_from_digit(n, NumkeyType::Function)
                    .unwrap_or(KeyCode::NoneKey);
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
            _                     => Key::from(KeyCode::NoneKey),
        }
    }
}

pub struct RustboxCanvas {
    root: RustBox,
    wants_close: bool,
}

impl RustboxCanvas {
    pub fn new(_display_size: Point<i32>, _window_title: &str) -> RustboxCanvas {

        let root = match RustBox::init(rustbox::InitOptions {
            output_mode: rustbox::OutputMode::EightBit,
            ..Default::default()
        }) {
            Result::Ok(v) => v,
            Result::Err(e) => panic!("{}", e),
        };

        RustboxCanvas {
            root: root,
            wants_close: false,
        }
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
                Event::KeyEvent(key) => keys.push(Key::from(key)),
                Event::NoEvent => (),
                _              => (),
            },
            Err(_) => (),
        }
        keys
    }

    fn print_glyph(&mut self, x: i32, y: i32, glyph: Glyph) {
        let rend_glyph = RenderableGlyph::from(glyph);
        self.root.print(x as usize,
                        y as usize,
                        rustbox::RB_NORMAL,
                        Color::White,
                        Color::Black,
                        &rend_glyph.ch.to_string())
    }

    fn close_window(&mut self) {
        self.wants_close = true;
    }

    fn window_closed(&self) -> bool {
        self.wants_close
    }
}

