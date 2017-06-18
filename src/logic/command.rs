use std::fmt::Display;

use calx_ecs::Entity;

use GameContext;
use data::Walkability;
use engine::keys::{Key, KeyCode};
use ecs::traits::*;
use graphics::cell::{CellFeature, StairDest, StairDir, StairKind};
use logic::Action;
use logic::entity::EntityQuery;
use infinigen::ChunkedWorld;
use point::{Direction, Point};
use prefab::PrefabMarker;
use state::GameState;
use world::traits::*;
use world::{self, World};

use super::debug_command::*;

pub type CommandResult<T> = Result<T, CommandError>;

pub enum CommandError {
    Bug(&'static str),
    Invalid(&'static str),
    Debug(String),
    Cancel,
}

/// A bindable command that can be executed by the player.
pub enum Command {
    Move(Direction),
    UseStairs(StairDir),
    Look,
    Pickup,
    Drop,
    Inventory,
    Wait,
    Quit,

    DebugMenu,
    Teleport,
}

impl From<Key> for Command {
    fn from(key: Key) -> Command {
        match key {
            Key { code: KeyCode::Escape, .. } => Command::Quit,
            Key { code: KeyCode::Left, .. } |
            Key { code: KeyCode::H, .. } |
            Key { code: KeyCode::NumPad4, .. } => Command::Move(Direction::W),
            Key { code: KeyCode::Right, .. } |
            Key { code: KeyCode::L, .. } |
            Key { code: KeyCode::NumPad6, .. } => Command::Move(Direction::E),
            Key { code: KeyCode::Up, .. } |
            Key { code: KeyCode::K, .. } |
            Key { code: KeyCode::NumPad8, .. } => Command::Move(Direction::N),
            Key { code: KeyCode::Down, .. } |
            Key { code: KeyCode::J, .. } |
            Key { code: KeyCode::NumPad2, .. } => Command::Move(Direction::S),
            Key { code: KeyCode::B, .. } |
            Key { code: KeyCode::NumPad1, .. } => Command::Move(Direction::SW),
            Key { code: KeyCode::N, .. } |
            Key { code: KeyCode::NumPad3, .. } => Command::Move(Direction::SE),
            Key { code: KeyCode::Y, .. } |
            Key { code: KeyCode::NumPad7, .. } => Command::Move(Direction::NW),
            Key { code: KeyCode::U, .. } |
            Key { code: KeyCode::NumPad9, .. } => Command::Move(Direction::NE),

            Key { code: KeyCode::Period, .. } => Command::UseStairs(StairDir::Ascending),
            Key { code: KeyCode::Comma, .. } => Command::UseStairs(StairDir::Descending),

            Key { code: KeyCode::M, .. } => Command::Look,
            Key { code: KeyCode::G, .. } => Command::Pickup,
            Key { code: KeyCode::D, .. } => Command::Drop,
            Key { code: KeyCode::I, .. } => Command::Inventory,

            Key { code: KeyCode::E, .. } => Command::Teleport,
            Key { code: KeyCode::F1, .. } => Command::DebugMenu,

            _ => Command::Wait,
        }
    }
}

pub fn process_player_command(context: &mut GameContext, command: Command) -> CommandResult<()> {
    match command {
        // TEMP: Commands can still be run even if there is no player?
        Command::Quit => Err(CommandError::Invalid("Can't quit.")),

        Command::Look => cmd_look(context),
        Command::UseStairs(dir) => cmd_use_stairs(context, dir),
        Command::Pickup => cmd_pickup(context),
        Command::Drop => cmd_drop(context),
        Command::Inventory => cmd_inventory(context),

        Command::Move(dir) => cmd_player_move(context, dir),
        Command::Wait => cmd_add_action(context, Action::Wait),

        Command::DebugMenu => cmd_debug_menu(context),
        Command::Teleport => cmd_teleport(context),
    }
}

fn cmd_player_move(context: &mut GameContext, dir: Direction) -> CommandResult<()> {
    // Check if we're bumping into something interactive, and if so don't consume a turn.
    let position = player_pos(context)?;
    let new_pos = position + dir;
    let npc_opt = context.state.world.find_entity(
        new_pos,
        |e| context.state.world.is_npc(*e),
    );
    if let Some(npc) = npc_opt {
        mes!(
            context.state.world,
            "{}: Hello!",
            a = npc.name(&context.state.world)
        );
        return Ok(());
    }

    cmd_add_action(context, Action::MoveOrAttack(dir))
}

fn cmd_add_action(context: &mut GameContext, action: Action) -> CommandResult<()> {
    context.state.add_action(action);
    Ok(())
}

fn cmd_look(context: &mut GameContext) -> CommandResult<()> {
    select_tile(context, maybe_examine_tile).map(|_| ())
}

fn cmd_teleport(context: &mut GameContext) -> CommandResult<()> {
    mes!(context.state.world, "Teleport where?");
    let pos = select_tile(context, |_, _| ())?;

    if context.state.world.can_walk(
        pos,
        Walkability::MonstersBlocking,
    )
    {
        cmd_add_action(context, Action::Teleport(pos))
    } else {
        Err(CommandError::Invalid("The way is blocked."))
    }
}

fn cmd_pickup(context: &mut GameContext) -> CommandResult<()> {
    let first_item;
    {
        let world = &context.state.world;
        let pos = player_pos(context)?;
        first_item = world.find_entity(pos, |&e| world.ecs().items.has(e))
    }

    match first_item {
        Some(item) => cmd_add_action(context, Action::Pickup(item)),
        None => Err(CommandError::Invalid("You grab at air.")),
    }
}

fn cmd_drop(context: &mut GameContext) -> CommandResult<()> {
    let player = context.state.world.player().ok_or(CommandError::Bug(
        "No player in the world!",
    ))?;
    let items = context.state.world.entities_in(player);
    let names = items.iter().map(|i| i.name(&context.state.world)).collect();
    let idx = menu_choice(context, names).ok_or(CommandError::Cancel)?;
    cmd_add_action(context, Action::Drop(items[idx]))
}

fn cmd_inventory(context: &mut GameContext) -> CommandResult<()> {
    let player = context.state.world.player().ok_or(CommandError::Bug(
        "No player in the world!",
    ))?;
    let items = context.state.world.entities_in(player);
    let names = items.into_iter()
        .map(|i| {
            context.state.world.ecs().names.get(i).unwrap().name.clone()
        })
        .collect();
    let choose = menu_choice_indexed(context, names)?;
    mes!(context.state.world, "You chose: {}", a = choose);
    Err(CommandError::Cancel)
}

fn find_stair_dest(state: &GameState, pos: Point, dir: StairDir) -> CommandResult<StairDest> {
    let cell = state.world.cell_const(&pos).ok_or(CommandError::Bug(
        "World was not loaded at stair pos!",
    ))?;

    match cell.feature {
        Some(CellFeature::Stairs(stair_dir, dest)) => {
            if stair_dir != dir {
                return Err(CommandError::Cancel);
            }

            debug!(state.world.logger, "STAIR at {}: {:?}", pos, dest);

            Ok(dest)
        },
        _ => Err(CommandError::Cancel),
    }
}

fn player_pos(context: &GameContext) -> CommandResult<Point> {
    let world = &context.state.world;
    let player = world.player().ok_or(
        CommandError::Bug("No player in the world!"),
    )?;
    let pos = world.position(player).ok_or(CommandError::Bug(
        "Player has no position!",
    ))?;
    Ok(pos)
}

fn cmd_use_stairs(context: &mut GameContext, dir: StairDir) -> CommandResult<()> {
    let pos = player_pos(context)?;
    let state = &mut context.state;
    let next = find_stair_dest(state, pos, dir)?;

    let (true_next, dest) = load_stair_dest(state, pos, next)?;
    state.world.move_to_map(true_next, dest).unwrap();

    debug!(state.world.logger, "map id: {:?}", state.world.map_id());
    Ok(())
}

fn load_stair_dest(
    state: &mut GameState,
    stair_pos: Point,
    next: StairDest,
) -> CommandResult<(World, Point)> {
    match next {
        StairDest::Generated(map_id, dest) => {
            let world = &mut state.world;
            debug!(world.logger, "Found stair leading to: {:?}", map_id);
            let world = world::serial::load_world(map_id).map_err(|_| {
                CommandError::Bug("Failed to load already generated world!")
            })?;
            Ok((world, dest))
        },
        StairDest::Ungenerated(stair_kind) => {
            debug!(state.world.logger, "Failed to load map, generating...");

            let res = {
                generate_stair_dest(state, stair_pos, stair_kind)
            };
            debug!(
                state.world.logger,
                "new stairs: {:?}",
                state.world.cell_const(&stair_pos)
            );
            res
        },
    }
}

fn generate_dungeon_floor(state: &mut GameState, dungeon_entity: Entity) -> CommandResult<World> {
    let dungeon = state.globals.ecs.dungeons.get_mut(dungeon_entity).expect("Invalid dungeon!");
    debug!(state.world.logger, "Dungeon status: {:?}", dungeon);
    let mut new_floor = dungeon.data.generate(&state.world)
        .ok_or(CommandError::Bug("Failed to generate stair!"))?;

    // now, connect the stairs to the next floor
    for (pos, marker) in new_floor.terrain().markers.clone().iter() {
        debug!(new_floor.logger, "M: {} {:?}", pos, marker);
        if *marker == PrefabMarker::StairsOut {
            debug!(new_floor.logger, "Connecting stairs to entity {:?}", dungeon_entity);
            let mut stairs_mut = new_floor.cell_mut(&pos).unwrap();

            if let Some(CellFeature::Stairs(_, StairDest::Ungenerated(ref mut kind))) =
                stairs_mut.feature {
                    *kind = StairKind::Dungeon(dungeon_entity);
                }
        }
    }

    Ok(new_floor)
}

fn generate_stair_dest(state: &mut GameState, stair_pos: Point, stair_kind: StairKind) -> CommandResult<(World, Point)> {
    let mut new_world = match stair_kind {
        StairKind::Dungeon(dungeon_entity) => {
            generate_dungeon_floor(state, dungeon_entity)?
        },
        StairKind::Blank => {
            return Err(CommandError::Bug("Stair was left in an unconnected state!"));
        }
    };

    let prev_id = state.world.flags().map_id;
    let dest_id = new_world.flags().map_id;

    let mut stairs_mut = state.world.cell_mut(&stair_pos).unwrap();

    if let Some(CellFeature::Stairs(stair_dir, ref mut dest @ StairDest::Ungenerated(..))) =
        stairs_mut.feature
    {
        let new_stair_pos = new_world.find_stairs_in().ok_or(CommandError::Bug(
            "Generated world has no stairs!",
        ))?;

        *dest = StairDest::Generated(dest_id, new_stair_pos);

        new_world.place_stairs(stair_dir.reverse(), new_stair_pos, prev_id, stair_pos);

        Ok((new_world, new_stair_pos))
    } else {
        Err(CommandError::Bug(
            "Stairs should have already been found by now...",
        ))
    }
}

use glium::glutin::{VirtualKeyCode, ElementState};
use glium::glutin;
use graphics::Color;
use point::LineIter;
use renderer;

fn maybe_examine_tile(pos: Point, world: &mut World) {
    if let Some(mob) = world.mob_at(pos) {
        if let Some(player) = world.player() {
            if player.can_see_other(mob, world) {
                mes!(world, "You see here a {}.", a = mob.name(world));
            }
        }
    }
}

fn draw_targeting_line(player_pos: Option<Point>, world: &mut World) {
    let camera = world.flags().camera;

    if let Some(player_pos) = player_pos {
        draw_line(player_pos, camera, world);
    }
}

fn draw_line(start: Point, end: Point, world: &mut World) {
    world.marks.clear();
    for pos in LineIter::new(start, end) {
        world.marks.add(pos, Color::new(255, 255, 255));
    }
    world.marks.add(end, Color::new(255, 255, 255));
}

/// Allow the player to choose a tile.
pub fn select_tile<F>(context: &mut GameContext, callback: F) -> CommandResult<Point>
    where
    F: Fn(Point, &mut World),
{
    let mut selected = false;
    let mut result = context.state.world.flags().camera;
    let player_pos = context.state
        .world
        .player()
        .map(|p| context.state.world.position(p))
        .unwrap_or(None);

    renderer::with_mut(|rc| {
        draw_targeting_line(player_pos, &mut context.state.world);
        rc.update(&context.state);

        rc.start_loop(|renderer, event| {
            match event {
                glutin::Event::KeyboardInput(ElementState::Pressed, _, Some(code)) => {
                    println!("Key: {:?}", code);
                    {
                        let world = &mut context.state.world;
                        match code {
                            VirtualKeyCode::Up => world.flags_mut().camera.y -= 1,
                            VirtualKeyCode::Down => world.flags_mut().camera.y += 1,
                            VirtualKeyCode::Left => world.flags_mut().camera.x -= 1,
                            VirtualKeyCode::Right => world.flags_mut().camera.x += 1,
                            VirtualKeyCode::Escape => return Some(renderer::Action::Stop),
                            VirtualKeyCode::Return => {
                                selected = true;
                                return Some(renderer::Action::Stop);
                            },
                            _ => (),
                        }
                        let camera = world.flags().camera;
                        result = camera;
                        callback(camera, world);

                        draw_targeting_line(player_pos, world);
                    }

                    renderer.update(&context.state);

                },
                _ => (),
            }
            None
        });
    });


    context.state.world.marks.clear();

    if selected {
        Ok(result)
    } else {
        Err(CommandError::Cancel)
    }
}

use renderer::ui::layers::ChoiceLayer;

pub fn menu_choice(context: &mut GameContext, choices: Vec<String>) -> Option<usize> {
    renderer::with_mut(|rc| {
        rc.update(&context.state);

        rc.query(&mut ChoiceLayer::new(choices))
    })
}

pub fn menu_choice_indexed<T: Display + Clone>(
    context: &mut GameContext,
    mut choices: Vec<T>,
) -> CommandResult<T> {
    let strings = choices.iter().cloned().map(|t| t.to_string()).collect();
    let idx = menu_choice(context, strings).ok_or(CommandError::Cancel)?;
    Ok(choices.remove(idx))
}

use renderer::ui::layers::InputLayer;

pub fn player_input(context: &mut GameContext, prompt: &str) -> Option<String> {
    renderer::with_mut(|rc| {
        rc.update(&context.state);

        rc.query(&mut InputLayer::new(prompt))
    })
}
