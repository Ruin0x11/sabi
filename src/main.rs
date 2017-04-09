extern crate backtrace;
extern crate chrono;
#[macro_use] extern crate lazy_static;
extern crate uuid;
extern crate rand;
extern crate serde;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate slog;
extern crate slog_stream;
pub extern crate tcod;
extern crate textwrap;
extern crate toml;
extern crate goap;

#[macro_use] extern crate enum_derive;
#[macro_use] extern crate macro_attr;

// #[cfg(feature = "with-rustbox")]
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
mod drawcalls;
mod fov;
mod logic;
mod event;
mod namegen;
mod stats;
mod util;
mod direction;
mod ui;

use slog::Logger;

use actor::*;
use engine::canvas;
use point::Point;
use state::GameState;

use keys::Keys;

pub struct GameContext {
    logger: Logger,
    state: GameState,
}

fn main() {
    run();
}

fn init() {
    log::init_panic_hook();
}

pub fn get_context() -> GameContext {
    GameContext {
        logger: log::make_logger("main").unwrap(),
        state: GameState::new(),
    }
}

pub fn run() {
    init();
    setup();
    // let mut context = get_context();
    // game_loop(&mut context);
    // info!(context.logger, "Exited cleanly.");
}

fn game_loop(mut ctxt: &mut GameContext) {
    while !canvas::window_closed() {
        state::process(ctxt);
    }
}


use world::*;
use rand::distributions::{Range, IndependentSample};
use testbed::start_with_params;

fn get_world() -> World {
    let mut world = World::generate(WorldType::Instanced(WorldPosition::new(64, 64)),
                                    16, tile::WALL);
    world.draw_square(WorldPosition::new(32, 32), 30, tile::FLOOR);
    world
}

fn setup() {
    let mut rng = rand::thread_rng();
    let mut world = get_world();

    let mut player = Actor::from_archetype(6, 6, "test_player");
    player.disposition = Disposition::Friendly;

    let range = Range::new(30, 200);

    for i in 0..16 {
        let mut other = Actor::from_archetype(10 + i, 16, "putit");
        other.speed = range.ind_sample(&mut rng);
        world.add_actor(other);
    }

    start_with_params(player, world);
}
