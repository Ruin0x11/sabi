use glyph::Glyph;

use std::error::Error;
use std::default::Default;

use rustbox::{self, Color, RustBox, Event};

use euclid::point::Point2D as Point;
use rustbox::Key as RustboxKey;

use engine::Canvas_;
use keys::{Key, KeyCode};

impl From<rustbox::Key> for Key {
    fn from(rb_key: rustbox::Key) -> Key {
        match rb_key {
            RustboxKey::Char(c) => Key {
                code: KeyCode::A,
                ..Default::default()
            },
            RustboxKey::Ctrl(c) => Key {
                code: KeyCode::A,
                ctrl: true,
                ..Default::default()
            },
            RustboxKey::F(f)    => Key {
                code: KeyCode::A,
                ..Default::default()
            },
            RustboxKey::Left    => Key::from(KeyCode::Left),
            RustboxKey::Right   => Key::from(KeyCode::Right),
            RustboxKey::Up      => Key::from(KeyCode::Up),
            RustboxKey::Down    => Key::from(KeyCode::Down),
            _                   => Key::from(KeyCode::A),
        }
    }
}

pub struct RustboxCanvas {
    root: RustBox,
    wants_close: bool,
}

impl RustboxCanvas {
    pub fn new(_display_size: Point<i32>, _window_title: &str) -> RustboxCanvas {

        let root = match RustBox::init(Default::default()) {
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

    fn print(&mut self, x: i32, y: i32, glyph: Glyph) {
        self.root.print(x as usize,
                        y as usize,
                        rustbox::RB_NORMAL,
                        Color::White,
                        Color::Black,
                        "@")
    }

    fn close_window(&mut self) {
        self.wants_close = true;
    }

    fn window_closed(&self) -> bool {
        self.wants_close
    }
}

