use std::collections::{HashMap, VecDeque};

use action::*;
use actor::*;
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
        self.run_action(action, id);
    }

    pub fn run_action(&mut self, action: Action, actor_id: Uuid) {
        let world_mut = self.current_world_mut();

        world_mut.run_action(action, actor_id);
    }
}

#[cfg(never)]
pub fn process_actors(context: &GameContext) {
    if let Some(ref mut world) = context.state.world {
        for actor in world.actors() {

        }
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

fn process_world(world: &mut World, canvas: &mut Canvas) {
    // TEMP: move to rendering area
    world.with_cells(Point::new(0, 0), Point::new(128, 128),
                     |point, ref cell| {
                         canvas.print_glyph(point.x, point.y, cell.tile.glyph.clone())
                     });
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

// TEMP: Just to bootstrap things dirtily.
pub fn process(context: &mut GameContext) {
    context.canvas.clear();

    // TEMP: Nothing to do with speed here!
    process_world(&mut context.state.world, &mut context.canvas);

    context.canvas.present();

    process_player(context);
}
