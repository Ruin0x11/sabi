#[macro_use]
extern crate bitflags;

extern crate backtrace;
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
mod tile;
mod chunk;
mod world;
mod point;
mod gen;
mod actor;
mod state;
mod testbed;

use slog::Logger;

use action::Action;
use actor::*;
use engine::Canvas;
use world::*;
use point::Point;
use state::GameState;

use keys::{Key, Keys, KeyCode};

pub struct GameContext {
    canvas: Canvas,
    logger: Logger,
    keys: Keys,
    state: GameState,
}

fn main() {
    run();
}

fn init() {
    log::init_panic_hook();
}

pub fn get_context() -> GameContext {
    let canvas = engine::get_canvas().unwrap();

    GameContext {
        canvas: canvas,
        logger: log::make_logger("main").unwrap(),
        keys: Keys::new(),
        state: GameState::new(),
    }
}

pub fn run() {
    init();
    let mut context = get_context();
    game_loop(&mut context);
}

fn game_loop(mut ctxt: &mut GameContext) {
    ctxt.state.set_world(World::generate(128, WorldType::Overworld));
    ctxt.state.set_player(Actor::new(5,5));

    while !ctxt.canvas.window_closed() {
        state::process(ctxt);
    }
}
