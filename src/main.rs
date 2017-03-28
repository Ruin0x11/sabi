extern crate backtrace;
extern crate chrono;
#[macro_use] extern crate lazy_static;
extern crate uuid;
extern crate rand;
#[macro_use] extern crate slog;
extern crate slog_stream;
pub extern crate tcod;
extern crate textwrap;

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
mod ai;
mod pathfinding;
mod turn_order;
mod drawcalls;
mod fov;
mod event;

use slog::Logger;

use actor::*;
use engine::Canvas;
use point::Point;
use state::GameState;

use keys::Keys;

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

    while !ctxt.canvas.window_closed() {
        state::process(ctxt);
    }
}
