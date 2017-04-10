use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};

use action::*;
use actor::*;
use direction::Direction;
use ai::{self};
use event;
use keys::*;
use point::Point;
use logic;
use world::{World, WorldType};
use engine::canvas;
use uuid::Uuid;
use ::GameContext;

pub struct GameState {
    pub world: World,
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

    pub fn add_action(&mut self, action: Action) {
        self.action_queue.push_back(action);
    }

    pub fn player_action(&mut self, action: Action) {
        let id = self.world.player_id();
        if !self.world.was_killed(&id) {
            logic::run_action(&mut self.world, &id, action);
        }
    }

    pub fn advance_time(& mut self, diff: i32) {
        let world = &mut self.world;
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

fn draw_overlays(world: &mut World) {
    world.draw_calls.draw_all();
    world.draw_calls.clear();
}

fn draw_world(world: &mut World) {
    let fov = world.player().fov();
    world.with_cells(Point::new(0, 0), Point::new(128, 128),
                     |point, ref cell| {
                         if fov.is_visible(&point) {
                             canvas::with(|c| c.print_glyph(point.x, point.y, cell.tile.glyph.clone()) )
                         }
                     });
}

fn show_messages(world: &mut World) {
    canvas::with_mut(|c| {
        let messages = world.pop_messages(c.width() as usize);
        debug!(world.logger, "Showing messages, len: {}", messages.len());
        c.update_message_buffer(messages)
    });
}

fn draw_items(world: &World) {
    let fov = world.player().fov();
    for item in world.items_in_map() {
        let pos = item.get_pos();
        if fov.is_visible(&pos) {
            canvas::with(|c| c.print_glyph(pos.x, pos.y, item.desc.glyph));
        }
    }
}

fn draw_actors(world: &mut World) {
    // TODO: Make trait for pos queryable?
    let fov = world.player().fov();
    for actor in world.actors() {
        let pos = actor.get_pos();
        if fov.is_visible(&pos) {
            canvas::with(|c| c.print_glyph(pos.x, pos.y, actor.glyph));
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

    let mut keys = canvas::with(|c| c.get_input());

    while let Some(key) = keys.pop() {
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

pub fn render_all(world: &mut World) {
    canvas::clear();
    let camera_pos = world.player().get_pos();
    canvas::with_mut(|c| c.set_camera(camera_pos.x, camera_pos.y));
    draw_world(world);
    draw_items(world);
    draw_actors(world);
    draw_overlays(world);
}

pub fn process_actors(world: &mut World) {
    while let Some(ref id) = world.next_actor() {
        let leftover_ticks = world.time_until_turn_for(id);
        if leftover_ticks > 0 {
            world.advance_time(leftover_ticks);
        }

        if world.is_player(id) {
            break
        }

        let action = {
            let actor = world.actor(id);
            ai::update_goal(actor, world);
            ai::update_memory(&actor, world);
            ai::choose_action(actor, world)
        };

        logic::run_action(world, id, action);

        process_events(world);
    }
}

pub fn check_player_dead(world: &mut World) -> bool {
    let id = world.player_id();
    let res = world.was_killed(&id);
    if res {
        info!(world.logger, "Player has died.");
        world.message("You're dead!".to_string());
        show_messages(world);
        canvas::present();
        canvas::get_input();
    }
    res
}

pub fn process_events(world: &mut World) {
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
    process_actors(&mut context.state.world);

    let dead = check_player_dead(&mut context.state.world);
    if dead {
        canvas::close_window();
        return;
    }

    render_all(&mut context.state.world);
    show_messages(&mut context.state.world);
    canvas::present();

    process_player_input(context);
}

pub fn step(context: &mut GameContext) {
    process_actors(&mut context.state.world);
    process_actors(&mut context.state.world);
}
