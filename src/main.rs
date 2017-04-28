#![feature(associated_consts)]
#![feature(test)]
#[macro_use] extern crate calx_ecs;
#[macro_use] extern crate enum_derive;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate macro_attr;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate slog;
extern crate backtrace;
extern crate bincode;
extern crate chrono;
extern crate goap;
extern crate infinigen;
extern crate noise;
extern crate rand;
extern crate serde;
extern crate slog_stream;
extern crate toml;
extern crate uuid;
pub extern crate tcod;

extern crate test;

// #[cfg(feature = "with-rustbox")]
extern crate rustbox;

#[cfg(feature = "with-opengl")]
#[macro_use]
extern crate glium;

// Macros must be used before all other modules
#[macro_use] mod log;

mod action;
mod ai;
mod cell;
mod chunk;
mod color;
mod command;
mod data;
mod direction;
mod drawcalls;
mod ecs;
mod engine;
mod fov;
mod glyph;
mod keys;
mod logic;
mod namegen;
mod pathfinding;
mod point;
mod state;
mod stats;
mod testbed;
mod ui;
mod util;
mod world;

#[cfg(test)]
mod testing;

use slog::Logger;

use world::traits::Mutate;
use engine::canvas;
use point::Point;
use state::GameState;

pub struct GameContext {
    logger: Logger,
    state: GameState,
}

impl GameContext {
    pub fn new() -> Self {
        GameContext {
            logger: log::make_logger("main"),
            state: GameState::new(),
        }
    }
}

fn main() {
    run();
}

fn init() {
    log::init_panic_hook();
}

pub fn run() {
    init();

    game_loop();

    println!("Done.");
}

fn game_loop() {
    let mut context = state::load_context();

    while !canvas::window_closed() {
        state::game_step(&mut context);
    }

    world::serial::save_world(&mut context.state.world).unwrap();
}
