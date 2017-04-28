use std::path::PathBuf;

use point::Point;
use slog::Logger;
use tcod::{self, Console, FontLayout, FontType, RootConsole};
use tcod::input::Key as TcodKey;
use tcod::input::KeyCode as TcodCode;

use engine::{self, Canvas_};
use glyph::{self, Glyph};
use keys::{self, Key, KeyCode};
use log;

use graphics::color::Color;

// bitflags! {
//     pub flags Attrs: u8 {
//         const ATTR_BOLD      = 0b00000001,
//         const ATTR_UNDERLINE = 0b00000010,
//         const ATTR_REVERSE   = 0b00000100,
//     }
// }

impl Into<tcod::Color> for Color {
    fn into(self) -> tcod::Color {
        tcod::Color {
            r: self.r,
            g: self.g,
            b: self.b,
        }
    }
}

pub struct TcodCanvas {
    logger: Logger,
    root: RootConsole,
    wants_close: bool,

    camera_x: i32,
    camera_y: i32,
}

impl TcodCanvas {
    pub fn new(display_size: Point,
               window_title: &str) -> TcodCanvas {
        let font_path = PathBuf::from("./fonts/terminal.png");
        let color = tcod::Color::new(0, 0, 0);

        let mut root = RootConsole::initializer()
            .title(window_title)
            .size(display_size.x, display_size.y)
            .font(font_path, FontLayout::AsciiInCol)
            .font_type(FontType::Greyscale)
            .init();
        root.set_default_background(color);

        TcodCanvas {
            logger: log::make_logger("graphics"),
            root: root,
            wants_close: false,

            camera_x: 0,
            camera_y: 0,
        }
    }
}

fn key_from_tcod(tcod_key: TcodKey) -> Option<Key> {
    let key_code = match tcod_key.code {
        TcodCode::Left      => Some(KeyCode::Left),
        TcodCode::Right     => Some(KeyCode::Right),
        TcodCode::Down      => Some(KeyCode::Down),
        TcodCode::Up        => Some(KeyCode::Up),

        TcodCode::Enter     => Some(KeyCode::Enter),
        TcodCode::Spacebar  => Some(KeyCode::Space),
        TcodCode::Escape    => Some(KeyCode::Esc),

        TcodCode::Tab       => Some(KeyCode::Tab),
        TcodCode::Backspace => Some(KeyCode::Backspace),
        TcodCode::Delete    => Some(KeyCode::Delete),
        TcodCode::Insert    => Some(KeyCode::Insert),

        TcodCode::Home      => Some(KeyCode::Home),
        TcodCode::End       => Some(KeyCode::End),
        TcodCode::PageUp    => Some(KeyCode::PageUp),
        TcodCode::PageDown  => Some(KeyCode::PageDown),

        TcodCode::Number0   => Some(KeyCode::D0),
        TcodCode::Number1   => Some(KeyCode::D1),
        TcodCode::Number2   => Some(KeyCode::D2),
        TcodCode::Number3   => Some(KeyCode::D3),
        TcodCode::Number4   => Some(KeyCode::D4),
        TcodCode::Number5   => Some(KeyCode::D5),
        TcodCode::Number6   => Some(KeyCode::D6),
        TcodCode::Number7   => Some(KeyCode::D7),
        TcodCode::Number8   => Some(KeyCode::D8),
        TcodCode::Number9   => Some(KeyCode::D9),

        TcodCode::NumPad0   => Some(KeyCode::NumPad0),
        TcodCode::NumPad1   => Some(KeyCode::NumPad1),
        TcodCode::NumPad2   => Some(KeyCode::NumPad2),
        TcodCode::NumPad3   => Some(KeyCode::NumPad3),
        TcodCode::NumPad4   => Some(KeyCode::NumPad4),
        TcodCode::NumPad5   => Some(KeyCode::NumPad5),
        TcodCode::NumPad6   => Some(KeyCode::NumPad6),
        TcodCode::NumPad7   => Some(KeyCode::NumPad7),
        TcodCode::NumPad8   => Some(KeyCode::NumPad8),
        TcodCode::NumPad9   => Some(KeyCode::NumPad9),

        TcodCode::F1        => Some(KeyCode::F1),
        TcodCode::F2        => Some(KeyCode::F2),
        TcodCode::F3        => Some(KeyCode::F3),
        TcodCode::F4        => Some(KeyCode::F4),
        TcodCode::F5        => Some(KeyCode::F5),
        TcodCode::F6        => Some(KeyCode::F6),
        TcodCode::F7        => Some(KeyCode::F7),
        TcodCode::F8        => Some(KeyCode::F8),
        TcodCode::F9        => Some(KeyCode::F9),
        TcodCode::F10       => Some(KeyCode::F10),
        TcodCode::F11       => Some(KeyCode::F11),
        TcodCode::F12       => Some(KeyCode::F12),

        TcodCode::Char      => Some(KeyCode::from(tcod_key.printable)),

        _ => None,
    };

    key_code.map(|code| Key {
        code: code,
        alt: tcod_key.alt,
        ctrl: tcod_key.ctrl,
        shift: tcod_key.shift,
    })
}

impl Canvas_ for TcodCanvas {
    fn print_info(&self) {
        println!("Tcod, to go!");
    }

    fn clear(&mut self) {
        let color = tcod::Color { r: 0, g: 0, b: 0 }; // IMPLEMENT
        self.root.set_default_foreground(color);
        self.root.clear();
    }

    fn present(&mut self) {
        self.root.flush();
    }

    fn set_camera(&mut self, x: i32, y: i32) {
        self.camera_x = x;
        self.camera_y = y;
    }

    fn translate_pos(&self, world_x: i32, world_y: i32) -> (i32, i32) {
        let w = self.width();
        let h = self.height();
        (world_x - self.camera_x + (w / 2), world_y - self.camera_y + (h / 2))
    }

    fn get_input(&self) -> Vec<Key> {
        let mut keys = Vec::new();
        while let Some(keycode) = self.root.check_for_keypress(tcod::input::KEY_PRESSED) {
            if let Some(key) = key_from_tcod(keycode) {
                keys.push(key);
            }
        }
        keys
    }


    fn print_str(&mut self, x: i32, y: i32, s: &str) {
        self.root.print(x, y, s);
    }

    fn print_glyph(&mut self, x: i32, y: i32, glyph: Glyph) {
        let (x, y) = self.translate_pos(x, y);
        if !engine::point_inside_canvas(self, Point::new(x, y)) {
            return;
        }
        let rend_glyph = glyph::lookup_ascii(glyph);
        let color_fg = rend_glyph.color_fg.clone().into();
        let color_bg = rend_glyph.color_bg.clone().into();

        self.root.set_char(x, y, rend_glyph.ch);
        self.root.set_char_foreground(x, y, color_fg);
        self.root.set_char_background(x, y, color_bg, tcod::BackgroundFlag::Set);
    }

    fn width(&self) -> i32 {
        self.root.width()
    }

    fn height(&self) -> i32 {
        self.root.height()
    }

    fn close_window(&mut self) {
        self.wants_close = true;
    }

    fn window_closed(&self) -> bool {
        self.wants_close || self.root.window_closed()
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
