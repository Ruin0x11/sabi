use std::collections::VecDeque;

use calx_ecs::Entity;
use infinigen::ChunkedWorld;

use ::GameContext;
use ai;
use chunk::generator::ChunkType;
use engine::canvas;
use graphics::Glyph;
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
        GameState {
            world: EcsWorld::new(Bounds::Unbounded, ChunkType::Blank, 1),
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

fn draw_world(world: &mut EcsWorld) {
    let size = canvas::size();

    let center = world.flags().camera - size/2;

    world.with_cells(center, size, |point, ref cell| {
        let visible = world.player().map_or(true, |p| {
            world.ecs().fovs.map_or(true, |f| f.is_visible(&point), p)
        });

        if true || visible {
            canvas::with(|c| c.print_glyph(point.x, point.y, cell.glyph()) )
        }
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
        if let Some(pos) = world.position(*e) {
            if let Some(a) = world.ecs().appearances.get(*e) {
                canvas::with(|c| c.print_glyph(pos.x, pos.y, a.glyph));
            }
        }
    }
}

fn get_player_command(context: &mut GameContext) -> Option<Command> {
    let mut keys = canvas::with(|c| c.get_input());

    let mut command = None;

    if let Some(key) = keys.pop() {
        info!(context.logger, "Key: {:?}", key);
        command = Some(Command::from_key(key));
    }
    command
}

fn process_player_command(context: &mut GameContext, command: Command) {
    match command {
        // TEMP: Commands can still be run even if there is no player?
        Command::Quit           => canvas::close_window(),
        Command::Move(dir)      => context.state.add_action(Action::MoveOrAttack(dir)),
        Command::Wait           => context.state.add_action(Action::Dood),
        Command::UseStairs(dir) => {command::try_use_stairs(dir, &mut context.state.world);},
    }
}

fn run_action_queue<'a>(context: &'a mut GameContext) {
    while let Some(action) = context.state.action_queue.pop_front() {
        context.state.player_action(action)
    }
}

fn render_world(world: &mut EcsWorld) {
    canvas::clear();
    let camera_pos = world.flags().camera;
    canvas::with_mut(|c| c.set_camera(camera_pos.x, camera_pos.y));
    draw_world(world);
    draw_entities(world);
    // draw_overlays(world);
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

        if !world.ecs().ais.has(entity) {
            if world.turn_order().contains(entity) {
                panic!("Entity without ai in turn order!");
            }
            continue;
        }

        let action = ai::run(entity, world);

        process_action(world, entity, action);

        process_events(world);
    }

    world.purge_dead();
}

fn check_player_dead(world: &mut EcsWorld) -> bool {
    let res = world.player().is_none();
    if res {
        info!(world.logger, "Player has died.");
        // world.message("You're dead!".to_string());
        // show_messages(world);
        canvas::present();
        canvas::get_input();
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
    //     //render_all(world, canvas);
    //     responses.extend(event::check_all(world));
    // }
}

fn update_world_terrain(world: &mut EcsWorld) {
    world.update_chunks().unwrap();
}

fn update_camera(world: &mut EcsWorld) {
    if let Some(player) = world.player() {
        if let Some(pos) = world.position(player) {
            world.flags_mut().camera = pos;
        }
    }
}

pub fn run_command(context: &mut GameContext, command: Command) {
    process_player_command(context, command);
    run_action_queue(context);
    process(context);
}

pub fn run_action(context: &mut GameContext, action: Action) {
    context.state.player_action(action);
    run_action_queue(context);
    process(context);
}

/// Treats all actors as frozen and only updates the world/chunk loading.
pub fn run_action_no_ai(context: &mut GameContext, action: Action) {
    context.state.player_action(action);
    run_action_queue(context);

    update_world_terrain(&mut context.state.world);
}

pub fn run_action_on(context: &mut GameContext, entity: Entity, action: Action) {
    process_action(&mut context.state.world, entity, action);
}

// TEMP: Just to bootstrap things dirtily.
pub fn process(context: &mut GameContext) {
    update_world_terrain(&mut context.state.world);

    process_actors(&mut context.state.world);
}

pub fn render(context: &mut GameContext) {
    update_camera(&mut context.state.world);

    render_world(&mut context.state.world);
    // show_messages(&mut context.state.world);
    canvas::present();
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
        let e = context.state.world.create(::ecs::prefab::mob("Player", 100000, Glyph::Player), WorldPosition::new(1,1));
        context.state.world.set_player(Some(e));
    }

    init(&mut context);
    context
}

pub fn init(context: &mut GameContext) {
    init_headless(context);
    render(context);
}

pub fn game_step(context: &mut GameContext) {
    if let Some(command) = get_player_command(context) {
        run_command(context, command);
    }
    render(context);

    let dead = check_player_dead(&mut context.state.world);
    if dead {
        canvas::close_window();
        return;
    }
}
