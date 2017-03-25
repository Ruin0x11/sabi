use std::collections::{HashMap, VecDeque};

use action::*;
use actor::*;
use ai::{self, Ai};
use keys::*;
use point::Point;
use world::{World, WorldType};
use engine::Canvas;
use uuid::Uuid;
use ::GameContext;

pub struct GameState {
    world: World,
    action_queue: VecDeque<Action>,
}

impl GameState {
    pub fn new() -> Self {
        GameState {
            world: World::new_empty(WorldType::Nothing, 0),
            action_queue: VecDeque::new(),
        }
    }

    pub fn set_world(&mut self, world: World) {
        self.world = world;
    }

    pub fn current_world(&self) -> &World {
        &self.world
    }

    pub fn current_world_mut(&mut self) -> &mut World {
        &mut self.world
    }

    pub fn add_action(&mut self, action: Action) {
        self.action_queue.push_back(action);
    }

    pub fn player_action(&mut self, action: Action) {
        let id = self.world.player_id();
        let world = self.current_world_mut();
        world.run_action(action, &id);
    }

    pub fn advance_time(&mut self, diff: i32) {
        let world = self.current_world_mut();
        world.advance_time(diff)
    }
}

pub enum Command {
    Move(Direction),
    Wait,
    Quit,
}

pub enum NextState {
    Inventory,
    Quit,
}

fn draw_overlays(world: &mut World, canvas: &mut Canvas) {
    world.draw_calls.draw_all(canvas);
}

fn draw_world(world: &mut World, canvas: &mut Canvas) {
    // TEMP: move to rendering area
    world.with_cells(Point::new(0, 0), Point::new(128, 128),
                     |point, ref cell| {
                         canvas.print_glyph(point.x, point.y, cell.tile.glyph.clone())
                     });
}

fn draw_actors(world: &mut World, canvas: &mut Canvas) {
    for actor in world.actors() {
        let pos = actor.get_pos();
        canvas.print_glyph(pos.x, pos.y, actor.glyph);
    }   
}

fn get_command_for_key(context: &GameContext, key: Key) -> Command {
    if key.code == KeyCode::NoneKey {
        warn!(context.logger, "NoneKey was returned");
    }
    debug!(context.logger, "Key: {:?}", key);
    match key {
        Key { code: KeyCode::Esc,   .. } => Command::Quit,
        Key { code: KeyCode::Left,  .. } => Command::Move(Direction::W),
        Key { code: KeyCode::Right, .. } => Command::Move(Direction::E),
        Key { code: KeyCode::Up,    .. } => Command::Move(Direction::N),
        Key { code: KeyCode::Down,  .. } => Command::Move(Direction::S),
        _                                => Command::Wait
    }
}

pub fn get_commands_from_input(context: &mut GameContext) -> Vec<Command> {
    let mut commands = Vec::new();

    let new_keys = context.canvas.get_input();
    context.keys.extend(new_keys);

    while let Some(key) = context.keys.pop() {
        commands.push(get_command_for_key(context, key));
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
        Command::Quit           => context.canvas.close_window(),
        Command::Move(dir)      => context.state.add_action(Action::Move(dir)),
        Command::Wait           => context.state.add_action(Action::Dood),
        _                       => ()
    }
}

fn process_player(context: &mut GameContext) {
    process_player_commands(context);

    while let Some(action) = context.state.action_queue.pop_front() {
        context.state.player_action(action)
    }
}

pub fn process_actors(world: &mut World) {
    while let Some(ref id) = world.next_actor() {
        if world.is_player(id) {
            break
        }

        let action = {
            let ai = ai::Simple;
            let actor = world.actor(id);
            ai.choose_action(actor, world)
        };

        world.run_action(action, id);
        world.advance_time(100);
    }
}

// TEMP: Just to bootstrap things dirtily.
pub fn process(context: &mut GameContext) {
    context.canvas.clear();

    draw_world(&mut context.state.world, &mut context.canvas);

    process_actors(&mut context.state.world);
    draw_actors(&mut context.state.world, &mut context.canvas);
    draw_overlays(&mut context.state.world, &mut context.canvas);

    context.canvas.present();

    process_player(context);
}
