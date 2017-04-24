#[macro_use] extern crate calx_ecs;
#[macro_use] extern crate enum_derive;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate macro_attr;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate slog;
extern crate backtrace;
extern crate chrono;
extern crate goap;
extern crate infinigen;
extern crate rand;
extern crate serde;
extern crate slog_stream;
extern crate toml;
extern crate uuid;
pub extern crate tcod;

// #[cfg(feature = "with-rustbox")]
extern crate rustbox;

#[cfg(feature = "with-opengl")]
#[macro_use]
extern crate glium;

mod action;
mod actor;
mod ai;
mod chunk;
mod color;
mod direction;
mod drawcalls;
mod engine;
mod event;
mod fov;
mod gen;
mod glyph;
mod keys;
mod log;
mod logic;
mod namegen;
mod pathfinding;
mod point;
mod state;
mod stats;
mod testbed;
mod tile;
mod ui;
mod util;
mod world;

mod ecs;
mod command;

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
    // setup();
    let mut context = get_context();
    game_loop(&mut context);
    // info!(context.logger, "Exited cleanly.");
    println!("Done.");
}

fn game_loop(mut ctxt: &mut GameContext) {
    use ecs::Mutate;
    let e = ctxt.state.world.create(::ecs::prefab::mob("Dood", ::glyph::Glyph::Player), Point::new(1,1));
    ctxt.state.world.set_player(Some(e));
    while !canvas::window_closed() {
        state::process(ctxt);
    }
}

use rand::distributions::{Range, IndependentSample};
use testbed::{get_world, start_with_params};

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
