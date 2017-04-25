use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};

use infinigen::Chunked;

use ::GameContext;
use action::*;
use ai;
use command::Command;
use direction::Direction;
use ecs::*;
use ecs::traits::*;
use engine::canvas;
use keys::*;
use logic;
use point::Point;
use uuid::Uuid;
use world::EcsWorld;

pub struct GameState {
    pub world: EcsWorld,
    action_queue: VecDeque<Action>,
}

impl GameState {
    pub fn new() -> Self {
        GameState {
            world: EcsWorld::new(0),
            action_queue: VecDeque::new(),
        }
    }

    pub fn add_action(&mut self, action: Action) {
        self.action_queue.push_back(action);
    }

    pub fn player_action(&mut self, action: Action) {
        if let Some(player) = self.world.player() {
            logic::run_action(&mut self.world, player, action);
        }
    }

    pub fn advance_time(& mut self, diff: i32) {
        let world = &mut self.world;
        // world.advance_time(diff)
    }
}

pub enum NextState {
    Inventory,
    Quit,
}

#[cfg(never)]
fn draw_overlays(world: &mut EcsWorld) {
    world.draw_calls.draw_all();
    world.draw_calls.clear();
}

fn draw_world(world: &mut EcsWorld) {
    // let fov = world.player().fov();
    let size = canvas::size();
    let center = world.flags().camera - size/2;

    world.with_cells(center, size, |point, ref cell| {
        canvas::with(|c| c.print_glyph(point.x, point.y, cell.glyph) )
    } );
}

#[cfg(never)]
fn show_messages(world: &mut EcsWorld) {
    canvas::with_mut(|c| {
        let messages = world.pop_messages(c.width() as usize);
        debug!(world.logger, "Showing messages, len: {}", messages.len());
        c.update_message_buffer(messages)
    });
}

fn draw_entities(world: &mut EcsWorld) {
    // TODO: Make trait for pos queryable?
    for e in world.entities() {
        let pos = world.position(*e);
        if let Some(pos) = world.position(*e) {
            if let Some(a) = world.ecs().appearances.get(*e) {
                canvas::with(|c| c.print_glyph(pos.x, pos.y, a.glyph));
            }
        }
    }
}

pub fn get_commands_from_input(context: &mut GameContext) -> Vec<Command> {
    let mut commands = Vec::new();

    let mut keys = canvas::with(|c| c.get_input());

    while let Some(key) = keys.pop() {
        info!(context.logger, "Key: {:?}", key);
        commands.push(Command::from_key( key));
    }
    commands
}

pub fn process_player_commands(context: &mut GameContext) {
    let commands = get_commands_from_input(context);

    if let Some(first) = commands.iter().next() {
        process_player_command(first, context);
    }
}

fn process_player_command(command: &Command, context: &mut GameContext) {
    match *command {
        // TEMP: Commands can still be run even if there is no player?
        Command::Quit           => canvas::close_window(),
        Command::Move(dir)      => context.state.add_action(Action::Move(dir)),
        Command::Wait           => context.state.add_action(Action::Dood),
        _                       => ()
    }
}

fn process_player_input<'a>(context: &'a mut GameContext) {
    process_player_commands(context);

    while let Some(action) = context.state.action_queue.pop_front() {
        context.state.player_action(action)
    }
}

pub fn render_all(world: &mut EcsWorld) {
    canvas::clear();
    let camera_pos = world.flags().camera;
    canvas::with_mut(|c| c.set_camera(camera_pos.x, camera_pos.y));
    draw_world(world);
    draw_entities(world);
    // draw_overlays(world);
}

pub fn process_actors(world: &mut EcsWorld) {
    while let Some(id) = world.next_entity() {
        let leftover_ticks = world.turn_order().get_time_for(&id);
        if leftover_ticks > 0 {
            world.advance_time(leftover_ticks);
        }

        if world.is_player(id) {
            break
        }

        if !world.is_alive(id) {
            panic!("Killed actor remained in turn order!");
        }

        if !world.ecs().ais.has(id) {
            continue;
        }

        let action = {
            ai::update_goal(id, world);
            ai::update_memory(&id, world);
            ai::choose_action(id, world)
        };

        logic::run_action(world, id, action);

        process_events(world);
    }
}

pub fn check_player_dead(world: &mut EcsWorld) -> bool {
    let res = world.player().is_none();
    if res {
        // info!(world.logger, "Player has died.");
        // world.message("You're dead!".to_string());
        // show_messages(world);
        canvas::present();
        canvas::get_input();
    }
    res
}

pub fn process_events(world: &mut EcsWorld) {
    // let mut responses = event::check_all(world);
    // while responses.len() != 0 {
    //     world.events.clear();
    //     while let Some((action, id)) = responses.pop() {
    //         // FIXME: don't delay actors here.
    //         logic::run_action(world, &id, action);
    //     }
    //     //render_all(world, canvas);
    //     responses.extend(event::check_all(world));
    // }
}

fn update_world_terrain(world: &mut EcsWorld) {
    world.update_chunks().unwrap();
}

fn update_camera(world: &mut EcsWorld) {
    if let Some(player) = world.flags().player {
        if let Some(pos) = world.position(player) {
            world.flags_mut().camera = pos;
        }
    }
}

// TEMP: Just to bootstrap things dirtily.
pub fn process(context: &mut GameContext) {
    update_world_terrain(&mut context.state.world);

    process_actors(&mut context.state.world);

    let dead = check_player_dead(&mut context.state.world);
    if dead {
        canvas::close_window();
        return;
    }

    update_camera(&mut context.state.world);

    render_all(&mut context.state.world);
    // show_messages(&mut context.state.world);
    canvas::present();

    process_player_input(context);
}

pub fn step(context: &mut GameContext) {
    process_actors(&mut context.state.world);
    process_actors(&mut context.state.world);
}
