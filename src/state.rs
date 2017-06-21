use std::collections::VecDeque;

use calx_ecs::Entity;

use GameContext;
use ai;
use chunk::generator::ChunkType;
use ecs::globals::*;
use engine::keys::Key;
use graphics::cell::StairKind;
use infinigen::ChunkedWorld;
use logic::command::{self, Command, CommandError};
use logic::{self, Action};
use stats;
use terrain::traits::*;
use world::serial::SaveManifest;
use world::traits::*;
use world::{self, Bounds, World, WorldPosition};

pub struct GameState {
    pub world: World,
    pub globals: Globals,

    action_queue: VecDeque<Action>,
}

impl GameState {
    pub fn new() -> Self {
        let mut globals = Globals::new();
        globals.make_dungeon();
        globals.make_town();
        GameState {
            world: World::new()
                .with_bounds(Bounds::Unbounded)
                .with_chunk_type(ChunkType::Perlin)
                .with_randomized_seed()
                .build()
                .unwrap(),
            globals: globals,
            action_queue: VecDeque::new(),
        }
    }

    /// Try various things based on the world terrain being loaded at certain places, like
    /// inserting dungeon entrances onto the world.
    pub fn try_globals_update(&mut self) {
        if self.is_overworld() {
            debug!(self.world.logger, "This is apparently the overworld, updating globals...");
            for dungeon in self.globals.dungeons() {
                let stair_pos = dungeon.position(&self.globals).unwrap();
                let dungeon_compo = self.globals.ecs.dungeons.get_mut(dungeon).unwrap();
                if !dungeon_compo.placed {
                    if self.world.pos_loaded(&stair_pos) {
                        self.world
                            .place_stairs_down(stair_pos, StairKind::Dungeon(dungeon));
                    }
                    dungeon_compo.placed = true;
                }
            }

            for town in self.globals.towns() {
                let mut place = false;
                let town_pos = town.position(&self.globals).unwrap();
                {
                    let town_compo = self.globals.ecs.towns.get(town).unwrap();
                    if !town_compo.placed {
                        if town_compo.spanning_chunks(town_pos).iter().any(|ci| self.world.chunk_loaded(ci)) {
                            place = true;
                        }
                    }
                }

                if place {
                    let mut town_compo_mut = self.globals.ecs.towns.get_mut(town).unwrap();
                    let prefab = town_compo_mut.generate(town_pos);
                    town_compo_mut.size = prefab.size();
                    self.world.deploy_prefab(prefab, town_pos);
                    self.world.reify_markers();
                    town_compo_mut.placed = true;
                }
            }
        }
    }

    pub fn is_overworld(&self) -> bool {
        *self.world.terrain().bounds() == Bounds::Unbounded
    }

    pub fn clear_actions(&mut self) {
        self.action_queue.clear();
    }

    pub fn add_action(&mut self, action: Action) {
        self.action_queue.push_back(action);
    }

    pub fn player_action(&mut self, action: Action) {
        if let Some(player) = self.world.player() {
            process_action(&mut self.world, player, action);
        }
    }

    pub fn to_save_manifest(self) -> SaveManifest {
        SaveManifest {
            globals: self.world.flags().globals.clone(),
            globals_b: self.globals,
            map_id: self.world.map_id(),
        }
    }
}

pub fn game_step(context: &mut GameContext, input: Option<Key>) {
    if player_is_dead(&mut context.state.world) {
        restart_game(context);
        return;
    }

    if let Some(key) = input {
        let command = Command::from(key);
        run_command(context, command);
    }
}

pub fn run_command(context: &mut GameContext, command: Command) {
    match command::process_player_command(context, command) {
        Err(e) => {
            match e {
                CommandError::Bug(reason) => panic!("A bug occurred: {}", reason),
                CommandError::Debug(mes) => context.state.world.message(&mes),
                CommandError::Invalid(reason) => context.state.world.message(reason),
                CommandError::Cancel => (),
            }
            context.state.clear_actions();
            context.state.world.update_camera();
        },
        Ok(..) => {
            run_action_queue(context);
            process(context);
        },
    }
}


fn player_is_dead(world: &mut World) -> bool {
    let res = world.player().is_none();
    if res {
        info!(world.logger, "Player has died.");
        mes!(world, "You're dead!");
    }
    res
}

fn run_action_queue(context: &mut GameContext) {
    while let Some(action) = context.state.action_queue.pop_front() {
        context.state.player_action(action)
    }
}

fn process_action(world: &mut World, entity: Entity, action: Action) {
    logic::run_action(world, entity, action);

    if world.is_alive(entity) {
        let delay = stats::formulas::calculate_delay(world, entity, 100);
        world.add_delay_for(entity, delay);
    }
}

fn process_actors(world: &mut World) {
    // TODO: Allow events to also register themselves in the turn order, to allow them to execute
    // on time
    while let Some(entity) = world.next_entity_in_turn_order() {
        if !world.is_alive(entity) {
            panic!("Killed actor remained in turn order!");
        }

        let leftover_ticks = world.turn_order().get_time_for(entity).unwrap();
        if leftover_ticks > 0 {
            world.advance_time(leftover_ticks);
        }

        if world.is_player(entity) {
            // TODO: Check if any timed effects on the player need handling
            world.next_message();
            break;
        }

        if let Some(action) = ai::run(entity, world) {
            process_action(world, entity, action);
            // TODO: Check if any timed effects on this entity need handling
            process_events(world);
        }

        if player_is_dead(world) {
            break;
        }
    }

    world.purge_dead();
}

fn process_events(_world: &mut World) {
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


fn update_world(context: &mut GameContext) {
    context.state.world.update_terrain();
    context.state.world.update_camera();

    context.state.try_globals_update();
}

pub fn process(context: &mut GameContext) {
    update_world(context);
    process_actors(&mut context.state.world);
}

pub fn init_game_context(context: &mut GameContext) {
    context.state.world.on_load();
    context.state.try_globals_update();
    context.state.world.recalc_entity_fovs();
}

fn apply_save_manifest(context: &mut GameContext, mut world: World, manifest: SaveManifest) {
    world.flags_mut().globals = manifest.globals;
    context.state.world = world;
    context.state.globals = manifest.globals_b;
}

pub fn load_context() -> GameContext {
    let manifest = match world::serial::load_manifest() {
        Ok(m) => m,
        Err(_) => SaveManifest::new(),
    };

    let mut context = GameContext::new();

    if let Ok(world) = world::serial::load_world(manifest.map_id) {
        apply_save_manifest(&mut context, world, manifest);
    } else {
        let player = context.state
            .world
            .spawn(&::ecs::prefab::mob("player", 10000000, "player"),
                   WorldPosition::new(0, 0));
        context.state.world.set_player(Some(player));
    }

    init_game_context(&mut context);
    context
}

pub fn restart_game(context: &mut GameContext) {
    world::serial::wipe_save();
    *context = load_context();
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

    context.state.world.update_terrain();
}
