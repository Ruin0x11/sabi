#[macro_use]
extern crate bitflags;

extern crate chrono;
extern crate euclid;

#[macro_use]
extern crate slog;
extern crate slog_stream;

pub extern crate tcod;

#[cfg(feature = "with-rustbox")]
extern crate rustbox;

#[cfg(feature = "with-opengl")]
#[macro_use]
extern crate glium;

mod action;
mod color;
mod engine;
mod glyph;
mod keys;
mod log;

use std::panic;

use action::Action;
use color::*;
use engine::{Canvas};
use keys::{Key, Keys, KeyCode};
use slog::Logger;
use euclid::point::Point2D as Point;

pub struct Actor {
    x: i32,
    y: i32,
}

pub enum Direction {
    N,
    NE,
    E,
    SE,
    S,
    SW,
    W,
    NW,
}

impl Direction {
    fn to_movement_offset(&self) -> (i32, i32) {
        match *self {
            Direction::N  => (0,  -1),
            Direction::NE => (1,  -1),
            Direction::E  => (1,   0),
            Direction::SE => (1,   1),
            Direction::S  => (0,   1),
            Direction::SW => (-1,  1),
            Direction::W  => (-1,  0),
            Direction::NW => (-1, -1),
        }
    }
}

impl Actor {
    pub fn run_action(&mut self, action: Action) {
        match action {
            Action::Move(dir) => self.move_in_direction(dir),
            Action::Dood => println!("Dood!"),
        }
    }

    fn move_in_direction(&mut self, dir: Direction) {
        let (dx, dy) = dir.to_movement_offset();
        let cx = self.x.clone();
        let cy = self.y.clone();
        self.move_to(cx + dx, cy + dy);
    }

    fn move_to(&mut self, nx: i32, ny: i32) {
    // TODO: needs a map/world to check bounds, at minimum
        if true { 
            self.x = nx;
            self.y = ny;
        }
    }
}

pub struct GameContext<'a> {
    canvas: Canvas,
    logger: Logger,

    /// Actor that we take to be in control
    player: Option<&'a Actor>,
}

fn main() {

    let result = panic::catch_unwind(|| {
        run();
    });

    println!("{:?}", result);
}

fn run() {
    let canvas = engine::get_canvas().unwrap();

    let mut ctxt = GameContext {
        canvas: canvas,
        logger: log::make_logger("sabi").unwrap(),
        player: None,
    };

    let ctxt_mut = &mut ctxt;

    do_thing(ctxt_mut);
}

fn do_thing(mut ctxt: &mut GameContext) {
    let ref mut canvas = ctxt.canvas;

    let mut prayer = Actor { x: 0, y: 0 };

    let mut keys = Keys::new();
    while !canvas.window_closed() {
        canvas.clear();
        canvas.print_glyph(prayer.x, prayer.y, glyph::Glyph::Player);
        canvas.present();
        let new_keys = canvas.get_input();
        keys.extend(new_keys);
        if keys.matches(|k| k.code == KeyCode::Esc) {
            canvas.close_window();
        }
        while let Some(key) = keys.pop() {
            if key.code == KeyCode::NoneKey {
                warn!(ctxt.logger, "NoneKey was returned");
            }
            debug!(ctxt.logger, "Key: {:?}", key);
            let action = match key {
                Key { code: KeyCode::Left,  .. } => Action::Move(Direction::W),
                Key { code: KeyCode::Right, .. } => Action::Move(Direction::E),
                Key { code: KeyCode::Up,    .. } => Action::Move(Direction::N),
                Key { code: KeyCode::Down,  .. } => Action::Move(Direction::S),
                _ => Action::Dood,
            };
            prayer.run_action(action);
        }
    }
}
