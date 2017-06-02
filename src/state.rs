use std::collections::VecDeque;

use calx_ecs::Entity;
use rand::{self, Rng};

use ::GameContext;
use ai;
use chunk::generator::ChunkType;
use engine::keys::Key;
use point::POINT_ZERO;
use logic::command;
use logic::{self, Action, Command};
use stats;
use world::serial::SaveManifest;
use world::traits::*;
use world::{self, Bounds, EcsWorld, WorldPosition};

pub struct GameState {
    pub world: EcsWorld,
    action_queue: VecDeque<Action>,
}

impl GameState {
    pub fn new() -> Self {
        let seed = rand::thread_rng().next_u32();
        GameState {
            world: EcsWorld::new(Bounds::Unbounded, ChunkType::Perlin, seed, 0),
            action_queue: VecDeque::new(),
        }
    }

    pub fn add_action(&mut self, action: Action) {
        self.action_queue.push_back(action);
    }

    pub fn player_action(&mut self, action: Action) {
        if let Some(player) = self.world.player() {
            process_action(&mut self.world, player, action);
        }
    }
}

#[cfg(never)]
fn draw_overlays(world: &mut EcsWorld) {
    world.draw_calls.draw_all();
    world.draw_calls.clear();
}

#[cfg(never)]
fn show_messages(world: &mut EcsWorld) {
}

fn run_action_queue(context: &mut GameContext) {
    while let Some(action) = context.state.action_queue.pop_front() {
        context.state.player_action(action)
    }
}

fn process_action(world: &mut EcsWorld, entity: Entity, action: Action) {
    logic::run_action(world, entity, action);

    if world.is_alive(entity) {
        let delay = stats::formulas::calculate_delay(world, &entity, 100);
        world.add_delay_for(entity, delay);
    }
}

fn process_actors(world: &mut EcsWorld) {
    while let Some(entity) = world.next_entity() {
        if !world.is_alive(entity) {
            panic!("Killed actor remained in turn order!");
        }

        let leftover_ticks = world.turn_order().get_time_for(entity).unwrap();
        if leftover_ticks > 0 {
            world.advance_time(leftover_ticks);
        }

        if world.is_player(entity) {
            break
        }

        if let Some(action) = ai::run(entity, world) {
            process_action(world, entity, action);
            process_events(world);
        }
    }

    world.purge_dead();
}

fn check_player_dead(world: &EcsWorld) -> bool {
    let res = world.player().is_none();
    if res {
        info!(world.logger, "Player has died.");
        // world.message("You're dead!".to_string());
        // show_messages(world);
    }
    res
}

fn process_events(_world: &mut EcsWorld) {
    // let mut responses = event::check_all(world);
    // while responses.len() != 0 {
    //     world.events.clear();
    //     while let Some((action, id)) = responses.pop() {
    //         // FIXME: don't delay actors here.
    //         logic::run_action(world, &id, action);
    //     }
    //     responses.extend(event::check_all(world));
    // }
}

fn update_world_terrain(world: &mut EcsWorld) {
    let center = match world.player() {
        Some(p) => world.position(p).map_or(POINT_ZERO, |p| p),
        None    => POINT_ZERO,
    };

    world.update_chunks(center).unwrap();
}

fn update_camera(world: &mut EcsWorld) {
    if let Some(player) = world.player() {
        if let Some(pos) = world.position(player) {
            world.flags_mut().camera = pos;
        }
    }
}

pub fn run_command(context: &mut GameContext, command: Command) {
    command::process_player_command(context, command);
    run_action_queue(context);
    process(context);
}

// TEMP: Just to bootstrap things dirtily.
pub fn process(context: &mut GameContext) {
    update_world_terrain(&mut context.state.world);

    process_actors(&mut context.state.world);
}

pub fn init_headless(context: &mut GameContext) {
    update_world_terrain(&mut context.state.world);
}

pub fn load_context() -> GameContext {
    let manifest = match world::serial::load_manifest() {
        Ok(m) => m,
        Err(_) => SaveManifest::new(),
    };

    let mut context = GameContext::new();

    if let Ok(world) = world::serial::load_world(manifest.map_id) {
        context.state.world = world;
    } else {
        let e = context.state.world.create(::ecs::prefab::mob("Player", 10000, "player"), WorldPosition::new(1,1));
        context.state.world.set_player(Some(e));
    }

    init(&mut context);
    context
}

pub fn init(context: &mut GameContext) {
    init_headless(context);
}

pub fn game_step(context: &mut GameContext, input: Option<Key>) {
    if let Some(key) = input {
        let command = Command::from(key);
        run_command(context, command);
    }
    update_camera(&mut context.state.world);

    let dead = check_player_dead(&context.state.world);
    if dead {
        return;
    }
}


#[cfg(test)]
pub fn run_action(context: &mut GameContext, action: Action) {
    context.state.player_action(action);
    run_action_queue(context);
    process(context);
}

/// Treats all actors as frozen and only updates the world/chunk loading.
#[cfg(test)]
pub fn run_action_no_ai(context: &mut GameContext, action: Action) {
    context.state.player_action(action);
    run_action_queue(context);

    update_world_terrain(&mut context.state.world);
}
