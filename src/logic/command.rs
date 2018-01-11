use std::fmt::Display;

use uuid::Uuid;

use GameContext;
use data::Walkability;
use ecs::traits::*;
use engine::keys::{Key, KeyCode};
use glium::glutin::{VirtualKeyCode, ElementState};
use glium::glutin;
use graphics::color::{self, Color};
use graphics::cell::StairDir;
use logic::Action;
use logic::entity::EntityQuery;
use point::{Direction, Point, LineIter, SquareIter};
use renderer;
use renderer::ui::layers::*;
use world::traits::*;
use world::World;

use super::debug_command::*;
use super::stairs::*;

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
    Map,
    Wait,
    Quit,

    Zap,

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

            Key { code: KeyCode::M, .. } => Command::Map,
            Key { code: KeyCode::P, .. } => Command::Look,
            Key { code: KeyCode::G, .. } => Command::Pickup,
            Key { code: KeyCode::D, .. } => Command::Drop,
            Key { code: KeyCode::I, .. } => Command::Inventory,

            Key { code: KeyCode::Z, .. } => Command::Zap,

            Key { code: KeyCode::E, .. } => Command::Teleport,
            Key { code: KeyCode::F1, .. } => Command::DebugMenu,

            _ => Command::Wait,
        }
    }
}

pub fn process_player_command(context: &mut GameContext, command: Command) -> CommandResult<()> {
    match command {
        Command::Quit => Err(CommandError::Invalid("Can't quit.")),

        Command::Look => cmd_look(context),
        Command::UseStairs(dir) => cmd_use_stairs(context, dir),
        Command::Pickup => cmd_pickup(context),
        Command::Drop => cmd_drop(context),
        Command::Inventory => cmd_inventory(context),
        Command::Map => cmd_map(context),

        Command::Move(dir) => cmd_player_move(context, dir),
        Command::Wait => cmd_add_action(context, Action::Wait),

        Command::Zap => cmd_zap(context),

        Command::DebugMenu => cmd_debug_menu(context),
        Command::Teleport => cmd_teleport(context),
    }
}

fn quest_window(context: &mut GameContext, npc: Uuid) -> CommandResult<()> {
    let center = player_pos(context)?;
    let mut quests = super::quest::quests(npc);
    loop {
        let (map, size) = terrain_region(context, center, 32);
        let idx = renderer::with_mut(|renderer| {
            renderer.update(&context.state);
            renderer.query(&mut QuestLayer::new(quests.clone(), map, size))
        });

        if let Some(idx) = idx {
            quests.remove(idx);
        } else {
            break;
        }
    }

    Ok(())
}

fn cmd_player_move(context: &mut GameContext, dir: Direction) -> CommandResult<()> {
    let position = player_pos(context)?;
    let new_pos = position + dir;
    let mob_opt = context.state
                         .world
                         .find_entity(new_pos, |e| context.state.world.is_mob(*e));
    if let Some(mob) = mob_opt {
        // Check if we're bumping into an NPC, and if so don't consume a turn.
        if context.state.world.is_npc(mob) {
            let uuid = mob.uuid(&context.state.world).unwrap();
            return quest_window(context, uuid);
        }

        let player = context.state.world.player().unwrap();
        if mob.is_friendly(player, &context.state.world) {
            return cmd_add_action(context, Action::SwitchPlaces(mob));
        }
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

    if context.state
              .world
              .can_walk(pos, Walkability::MonstersBlocking)
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
    let player = context.state
                        .world
                        .player()
                        .ok_or(CommandError::Bug("No player in the world!"))?;
    let items = context.state.world.entities_in(player);
    let names = items.iter().map(|i| i.name(&context.state.world)).collect();

    let idx = menu_choice(context, names).ok_or(CommandError::Cancel)?;

    cmd_add_action(context, Action::Drop(items[idx]))
}

fn cmd_inventory(context: &mut GameContext) -> CommandResult<()> {
    let player = context.state
                        .world
                        .player()
                        .ok_or(CommandError::Bug("No player in the world!"))?;
    let items = context.state.world.entities_in(player);
    let names = items.iter().map(|i| i.name(&context.state.world)).collect();

    let choose = menu_choice_indexed(context, names)?;

    mes!(context.state.world, "You chose: {}", choose);
    Err(CommandError::Cancel)
}

fn terrain_region(context: &mut GameContext,
                  center: Point,
                  radius: i32)
                  -> (Vec<Color>, (u32, u32)) {
    let mut tiles = Vec::new();

    for point in SquareIter::new(center, radius) {
        let cell = context.state.world.cell(&point);
        let color = match cell {
            Some(c) => renderer::with(|r| r.cell_to_color(c)),
            None => color::BLACK,
        };
        tiles.push(color);
    }

    let true_radius = (tiles.len() as f32).sqrt() as u32;

    (tiles, (true_radius, true_radius))
}

fn cmd_map(context: &mut GameContext) -> CommandResult<()> {
    let center = player_pos(context)?;
    let (map, size) = terrain_region(context, center, 32);

    renderer::with_mut(|renderer| {
        renderer.update(&context.state);
        renderer.query(&mut MapLayer::new(map, size));
    });

    Ok(())
}

fn cmd_zap(context: &mut GameContext) -> CommandResult<()> {
    mes!(&context.state.world, "Zap where?");

    let dir = select_direction(context)?;

    cmd_add_action(context, Action::Missile(dir))
}

pub(super) fn player_pos(context: &GameContext) -> CommandResult<Point> {
    let world = &context.state.world;
    let player = world.player()
                      .ok_or(CommandError::Bug("No player in the world!"))?;
    let pos = world.position(player)
                   .ok_or(CommandError::Bug("Player has no position!"))?;
    Ok(pos)
}

fn maybe_examine_tile(pos: Point, world: &mut World) {
    if let Some(mob) = world.mob_at(pos) {
        if let Some(player) = world.player() {
            if player.can_see_other(mob, world) {
                format_mes!(world, player, "%u <see> here {}.", mob.name_with_article(world));
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

/// Allow the player to choose a direction.
pub fn select_direction(context: &mut GameContext) -> CommandResult<Direction> {
    let mut selected = false;
    let mut result = Direction::N;

    renderer::with_mut(|rc| {
        rc.update(&context.state);

        rc.start_loop(|renderer, event| {
            match event {
                glutin::Event::WindowEvent { event, .. } => {
                    match event {
                        glutin::WindowEvent::KeyboardInput { input, .. } => {
                            if ElementState::Pressed == input.state {
                                if let Some(code) = input.virtual_keycode {
                                    println!("Key: {:?}", code);
                                    {
                                        match code {
                                            VirtualKeyCode::Left | VirtualKeyCode::H => {
                                                result = Direction::W;
                                                selected = true;
                                            },
                                            VirtualKeyCode::Down | VirtualKeyCode::J => {
                                                result = Direction::S;
                                                selected = true;
                                            },
                                            VirtualKeyCode::Up | VirtualKeyCode::K => {
                                                result = Direction::N;
                                                selected = true;
                                            },
                                            VirtualKeyCode::Right | VirtualKeyCode::L => {
                                                result = Direction::E;
                                                selected = true;
                                            },
                                            VirtualKeyCode::Escape => {
                                                return Some(renderer::Action::Stop)
                                            },
                                            _ => (),
                                        }
                                    }
                                    renderer.update(&context.state);

                                    if selected {
                                        return Some(renderer::Action::Stop);
                                    }
                                }
                            }
                        },
                        _ => (),
                    }
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
                glutin::Event::WindowEvent { event, .. } => {
                    match event {
                        glutin::WindowEvent::KeyboardInput { input, .. } => {
                            if ElementState::Pressed == input.state {
                                if let Some(code) = input.virtual_keycode {
                                    println!("Key: {:?}", code);
                                    {
                                        let world = &mut context.state.world;
                                        match code {
                                            VirtualKeyCode::Up | VirtualKeyCode::K => {
                                                world.flags_mut().camera.y -= 1
                                            },
                                            VirtualKeyCode::Down | VirtualKeyCode::J => {
                                                world.flags_mut().camera.y += 1
                                            },
                                            VirtualKeyCode::Left | VirtualKeyCode::H => {
                                                world.flags_mut().camera.x -= 1
                                            },
                                            VirtualKeyCode::Right | VirtualKeyCode::L => {
                                                world.flags_mut().camera.x += 1
                                            },
                                            VirtualKeyCode::Escape => {
                                                return Some(renderer::Action::Stop)
                                            },
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
                                }
                            }
                        },
                        _ => (),
                    }
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

pub fn menu_choice(context: &mut GameContext, choices: Vec<String>) -> Option<usize> {
    renderer::with_mut(|renderer| {
        renderer.update(&context.state);
        renderer.query(&mut ChoiceLayer::new(choices))
    })
}

pub fn menu_choice_indexed<T: Display + Clone>(context: &mut GameContext,
                                               mut choices: Vec<T>)
                                               -> CommandResult<T> {
    let strings = choices.iter().cloned().map(|t| t.to_string()).collect();
    let idx = menu_choice(context, strings).ok_or(CommandError::Cancel)?;
    Ok(choices.remove(idx))
}

pub fn player_input(context: &mut GameContext, prompt: &str) -> Option<String> {
    renderer::with_mut(|renderer| {
        renderer.update(&context.state);
        renderer.query(&mut InputLayer::new(prompt))
    })
}
