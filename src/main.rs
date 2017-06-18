#![feature(associated_consts)]
#![feature(conservative_impl_trait)]
#![feature(test)]

#[macro_use]
extern crate calx_ecs;
#[macro_use]
extern crate enum_derive;
#[macro_use]
extern crate hlua;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate macro_attr;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate slog;

extern crate backtrace;
extern crate bincode;
extern crate calx_alg;
extern crate cgmath;
extern crate chrono;
extern crate crypto;
extern crate glob;
extern crate goap;
extern crate image;
extern crate infinigen;
extern crate noise;
extern crate rand;
extern crate regex;
extern crate rusttype;
extern crate serde;
extern crate slog_stream;
extern crate texture_packer;
extern crate toml;

extern crate test;

#[macro_use]
extern crate glium;

// Macros must be used before all other modules
#[macro_use]
mod macros;

mod ai;
mod chunk;
mod data;
mod ecs;
mod engine;
mod graphics;
mod item;
mod log;
mod logic;
mod lua;
mod point;
mod prefab;
mod renderer;
mod state;
mod stats;
mod terrain;
mod testbed;
mod util;
mod world;

#[cfg(test)]
mod testing;

use glium::glutin;
use glium::glutin::{VirtualKeyCode, ElementState};
use state::GameState;
use engine::keys::{Key, KeyCode};

pub struct GameContext {
    state: GameState,
}

impl GameContext {
    pub fn new() -> Self {
        GameContext { state: GameState::new() }
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
    renderer::with_mut(|rc| rc.update(&context.state));

    'outer: loop {
        let mut keys = Vec::new();
        let mut resize = None;
        let res = renderer::with(|rc| {
            for event in rc.poll_events() {
                match event {
                    glutin::Event::Closed => return false,
                    glutin::Event::Resized(w, h) => {
                        resize = Some((w, h));
                        continue;
                    },
                    _ => (),
                }

                match event {
                    glutin::Event::KeyboardInput(ElementState::Pressed, _, Some(code)) => {
                        match code {
                            VirtualKeyCode::Escape => return false,
                            _ => {
                                let key = Key::from(KeyCode::from(code));
                                keys.push(key);
                            },
                        }
                    },
                    _ => (),
                }
            }

            true
        });

        if !res {
            break 'outer;
        }

        for key in keys {
            state::game_step(&mut context, Some(key));
            renderer::with_mut(|renderer| renderer.update(&context.state));
        }

        renderer::with_mut(|renderer| {
            if let Some((w, h)) = resize {
                renderer.set_viewport(w, h)
            }

            renderer.render();
            renderer.step_frame();
        });
    }

    world::serial::save_world(&mut context.state.world).unwrap();
    world::serial::save_manifest(&context.state.world).unwrap();
}
