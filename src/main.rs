#[macro_use]
extern crate bitflags;

extern crate euclid;
extern crate rgb;

#[macro_use]
extern crate log;
extern crate env_logger;

pub extern crate tcod;

#[cfg(feature = "with-rustbox")]
extern crate rustbox;

#[cfg(feature = "with-opengl")]
#[macro_use]
extern crate glium;

mod glyph;
mod engine;
mod keys;

use engine::{Canvas};
use keys::{Key, Keys, KeyCode};

fn main() {
    env_logger::init().unwrap();
    
    let canvas = engine::get_canvas().unwrap();
    do_thing(canvas);
}

fn do_thing(mut canvas: Canvas) {
    let mut keys = Keys::new();
    let mut x: i32 = 1;
    let mut y: i32 = 1;
    while !canvas.window_closed() {
        canvas.clear();
        canvas.print(x, y, glyph::Glyph::Player);
        canvas.present();
        let new_keys = canvas.get_input();
        keys.extend(new_keys);
        if keys.matches(|k| k.code == KeyCode::A) {
            canvas.close_window();
        }
        while let Some(key) = keys.pop() {
            match key {
                Key { code: KeyCode::Left,  .. } => x -= 1,
                Key { code: KeyCode::Right, .. } => x += 1,
                Key { code: KeyCode::Up,    .. } => y -= 1,
                Key { code: KeyCode::Down,  .. } => y += 1,
                _ => (),
            }
            x = std::cmp::max(0, x);
            y = std::cmp::max(0, y);
        }
    }
}
