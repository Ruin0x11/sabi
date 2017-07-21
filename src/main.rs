#![feature(associated_consts)]
#![feature(conservative_impl_trait)]
#![feature(test)]

#[macro_use]
extern crate calx_ecs;

extern crate enum_derive;
#[macro_use]
extern crate hlua;
#[macro_use]
extern crate lazy_static;

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
extern crate rodio;
extern crate rusttype;
extern crate serde;
extern crate slog_stream;
extern crate texture_packer;
extern crate toml;
extern crate uuid;

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
mod sound;
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
    log::init_panic_hook();

    game_loop();

    println!("Exited cleanly.");
}

fn game_loop() {
    world::serial::init_save_paths().unwrap();

    let mut context = state::load_context();
    renderer::with_mut(|rc| rc.update(&context.state));

    'outer: loop {
        let mut keys = Vec::new();
        let mut resize = None;
        let mut quit = false;
        renderer::with_mut(|rc| {
            rc.poll_events(|event| match event {
                               glutin::Event::WindowEvent { event, .. } => {
                                   match event {
                                       glutin::WindowEvent::Closed => quit = true,
                                       glutin::WindowEvent::Resized(w, h) => {
                                           resize = Some((w, h));
                                       },
                                       _ => (),
                                   }

                                   match event {
                                       glutin::WindowEvent::KeyboardInput { input, .. } => {
                                           if let ElementState::Pressed = input.state {
                                               if let Some(code) = input.virtual_keycode {
                                                   match code {
                                                       VirtualKeyCode::Escape => quit = true,
                                                       _ => {
                                                           let key = Key::from(KeyCode::from(code));
                                                           keys.push(key);
                                                       },
                                                   }
                                               }
                                           }
                                       },
                                       _ => (),
                                   }
                               },
                               _ => (),
                           });

            false
        });

        if quit {
            break 'outer;
        }

        if let Some((w, h)) = resize {
            renderer::with_mut(|renderer| {
                renderer.set_viewport(w, h);
                renderer.update(&context.state);
                renderer.render();
            });
        }

        if let Some(key) = keys.first() {
            // Ensure that the renderer isn't borrowed during the game step, so it can be used in
            // the middle of any game routine (like querying the player for input)
            state::game_step(&mut context, Some(*key));

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
    world::serial::save_manifest(context.state).unwrap();
}
