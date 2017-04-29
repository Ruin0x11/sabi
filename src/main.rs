#![feature(associated_consts)]
#![feature(test)]

#[macro_use] extern crate calx_ecs;
#[macro_use] extern crate enum_derive;
#[macro_use] extern crate hlua;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate macro_attr;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate slog;

extern crate backtrace;
extern crate bincode;
extern crate calx_alg;
extern crate chrono;
extern crate goap;
extern crate infinigen;
extern crate noise;
extern crate rand;
extern crate serde;
extern crate slog_stream;
extern crate toml;
pub extern crate tcod;

extern crate test;

// #[cfg(feature = "with-rustbox")]
extern crate rustbox;

#[cfg(feature = "with-opengl")]
#[macro_use]
extern crate glium;

// Macros must be used before all other modules
#[macro_use] mod macros;

mod ai;
mod chunk;
mod data;
mod ecs;
mod engine;
mod graphics;
mod log;
mod logic;
mod lua;
mod point;
mod prefab;
mod state;
mod stats;
mod testbed;
mod ui;
mod util;
mod world;

#[cfg(test)]
mod testing;

use slog::Logger;

use engine::canvas;
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

    println!("Exited cleanly.");
}

fn game_loop() {
    world::serial::init_paths().unwrap();

    let mut context = state::load_context();

    while !canvas::window_closed() {
        state::game_step(&mut context);
    }

    world::serial::save_world(&mut context.state.world).unwrap();
    world::serial::save_manifest(&mut context.state.world).unwrap();
}
