use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};

use action::*;
use actor::*;
use direction::Direction;
use ai::{self, Ai};
use event;
use keys::*;
use point::Point;
use logic;
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
            // logic: Logic::new().
            // turn_order: TurnOrder::new(),
            // actors: Actors::new(),
            // 
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
        if !self.world.was_killed(&id) {
            let mut world = self.current_world_mut();
            logic::run_action(&mut world, &id, action);
        }
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
    world.draw_calls.clear();
}

fn draw_world(world: &mut World, canvas: &mut Canvas) {
    // FIXME: Let the player actor not be deleted, to get its fov.
    let fov = world.player().fov();
    world.with_cells(Point::new(0, 0), Point::new(128, 128),
                     |point, ref cell| {
                         if fov.is_visible(&point) {
                             canvas.print_glyph(point.x, point.y, cell.tile.glyph.clone())
                         }
                     });
}

fn show_messages(world: &mut World, canvas: &mut Canvas) {
    let messages = world.pop_messages(canvas.width() as usize);
    debug!(world.logger, "Showing messages, len: {}", messages.len());
    if messages.len() != 0 {
        canvas.show_messages(messages);
    }
}

fn draw_actors(world: &mut World, canvas: &mut Canvas) {
    // FIXME: Let the player actor not be deleted, to get its fov.
    let fov = world.player().fov();
    for actor in world.actors() {
        let pos = actor.get_pos();
        if fov.is_visible(&pos) {
            canvas.print_glyph(pos.x, pos.y, actor.glyph);
        }
    }
}

fn get_command_for_key(context: &GameContext, key: Key) -> Command {
    if let KeyCode::Unknown(c) = key.code {
        warn!(context.logger, "Unknown was returned, {}", c);
    }
    debug!(context.logger, "Key: {:?}", key);
    match key {
        Key { code: KeyCode::Esc,     .. } => Command::Quit,
        Key { code: KeyCode::Left,    .. } |
        Key { code: KeyCode::H,       .. } |
        Key { code: KeyCode::NumPad4, .. } => Command::Move(Direction::W),
        Key { code: KeyCode::Right,   .. } |
        Key { code: KeyCode::L,       .. } |
        Key { code: KeyCode::NumPad6, .. } => Command::Move(Direction::E),
        Key { code: KeyCode::Up,      .. } |
        Key { code: KeyCode::K,       .. } |
        Key { code: KeyCode::NumPad8, .. } => Command::Move(Direction::N),
        Key { code: KeyCode::Down,    .. } |
        Key { code: KeyCode::J,       .. } |
        Key { code: KeyCode::NumPad2, .. } => Command::Move(Direction::S),
        Key { code: KeyCode::NumPad1, .. } => Command::Move(Direction::SW),
        Key { code: KeyCode::NumPad3, .. } => Command::Move(Direction::SE),
        Key { code: KeyCode::NumPad7, .. } => Command::Move(Direction::NW),
        Key { code: KeyCode::NumPad9, .. } => Command::Move(Direction::NE),
        _                                  => Command::Wait
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

fn process_player_input(context: &mut GameContext) {
    process_player_commands(context);

    while let Some(action) = context.state.action_queue.pop_front() {
        context.state.player_action(action)
    }
}

pub fn render_all(world: &mut World, canvas: &mut Canvas) {
    canvas.clear();
    let camera_pos = world.player().get_pos();
    canvas.set_camera(camera_pos.x, camera_pos.y);
    draw_world(world, canvas);
    draw_actors(world, canvas);
    draw_overlays(world, canvas);
}

pub fn process_actors(world: &mut World, canvas: &mut Canvas) {
    while let Some(ref id) = world.next_actor() {
        let leftover_ticks = world.time_until_turn_for(id);
        if leftover_ticks > 0 {
            world.advance_time(leftover_ticks);
        }

        if world.is_player(id) {
            break
        }

        let action = {
            let ai = ai::Simple;
            let actor = world.actor(id);
            ai.choose_action(actor, world)
        };

        logic::run_action(world, id, action);

        process_events(world, canvas);
    }
}

pub fn check_player_dead(world: &mut World, canvas: &mut Canvas) -> bool {
    let id = world.player_id();
    let res = world.was_killed(&id);
    if res {
        info!(world.logger, "Player has died.");
        world.message("You're dead!".to_string());
        show_messages(world, canvas);
        canvas.present();
        canvas.get_input();
    }
    res
}

pub fn process_events(world: &mut World, canvas: &mut Canvas) {
    let mut responses = event::check_all(world);
    while responses.len() != 0 {
        world.events.clear();
        while let Some((action, id)) = responses.pop() {
            // FIXME: don't delay actors here.
            logic::run_action(world, &id, action);
        }
        //render_all(world, canvas);
        responses.extend(event::check_all(world));
    }
}

// TEMP: Just to bootstrap things dirtily.
pub fn process(context: &mut GameContext) {
    process_actors(&mut context.state.world, &mut context.canvas);

    let dead = check_player_dead(&mut context.state.world, &mut context.canvas);
    if dead {
        context.canvas.close_window();
        return;
    }

    render_all(&mut context.state.world, &mut context.canvas);
    show_messages(&mut context.state.world, &mut context.canvas);
    context.canvas.present();

    process_player_input(context);
}
