use ::GameContext;
use logic::Action;
use logic::entity;
use point::Direction;
use engine::keys::{Key, KeyCode};
use graphics::cell::{Cell, CellFeature, StairDest, StairDir};
use world::EcsWorld;
use point::Point;
use world::MapId;
use world::traits::*;

pub type CommandResult = Result<(), ()>;

pub enum Command {
    Move(Direction),
    UseStairs(StairDir),
    TestScript,
    Look,
    Wait,
    Quit,
}

impl From<Key> for Command {
    fn from(key: Key) -> Command {
        match key {
            Key { code: KeyCode::Escape,      .. } => Command::Quit,
            Key { code: KeyCode::Left,        .. } |
            Key { code: KeyCode::H,           .. } |
            Key { code: KeyCode::NumPad4,     .. } => Command::Move(Direction::W),
            Key { code: KeyCode::Right,       .. } |
            Key { code: KeyCode::L,           .. } |
            Key { code: KeyCode::NumPad6,     .. } => Command::Move(Direction::E),
            Key { code: KeyCode::Up,          .. } |
            Key { code: KeyCode::K,           .. } |
            Key { code: KeyCode::NumPad8,     .. } => Command::Move(Direction::N),
            Key { code: KeyCode::Down,        .. } |
            Key { code: KeyCode::J,           .. } |
            Key { code: KeyCode::NumPad2,     .. } => Command::Move(Direction::S),
            Key { code: KeyCode::B,           .. } |
            Key { code: KeyCode::NumPad1,     .. } => Command::Move(Direction::SW),
            Key { code: KeyCode::N,           .. } |
            Key { code: KeyCode::NumPad3,     .. } => Command::Move(Direction::SE),
            Key { code: KeyCode::Y,           .. } |
            Key { code: KeyCode::NumPad7,     .. } => Command::Move(Direction::NW),
            Key { code: KeyCode::U,           .. } |
            Key { code: KeyCode::NumPad9,     .. } => Command::Move(Direction::NE),

            Key { code: KeyCode::Period,      .. } => Command::UseStairs(StairDir::Ascending),
            Key { code: KeyCode::Comma,       .. } => Command::UseStairs(StairDir::Descending),

            Key { code: KeyCode::M,           .. } => Command::Look,

            Key { code: KeyCode::T,           .. } => Command::TestScript,

            _                                  => Command::Wait
        }
    }
}

pub fn process_player_command(context: &mut GameContext, command: Command) -> CommandResult {
    match command {
        // TEMP: Commands can still be run even if there is no player?
        Command::Quit           => (),
        Command::Move(dir)      => context.state.add_action(Action::MoveOrAttack(dir)),
        Command::Wait           => context.state.add_action(Action::Wait),
        Command::Look           => look(context),
        Command::TestScript     => context.state.add_action(Action::TestScript),
        Command::UseStairs(dir) => return try_use_stairs(dir, &mut context.state.world),
    }
    Ok(())
}

pub fn try_use_stairs(dir: StairDir, world: &mut EcsWorld) -> CommandResult {
    let player = world.player().ok_or(())?;
    let pos = world.position(player).ok_or(())?;
    let next = find_stair_dest(world, pos, dir)?;

    let (true_next, dest) = load_stair_dest(world, pos, next);
    world.move_to_map(true_next, dest).unwrap();

    debug!(world.logger, "map id: {:?}", world.map_id());
    Ok(())
}

fn find_stair_dest(world: &EcsWorld, pos: Point, dir: StairDir) -> Result<StairDest, ()> {
    let cell = match world.cell_const(&pos) {
        Some(c) => c,
        None    => return Err(())
    };

    match cell.feature {
        Some(CellFeature::Stairs(stair_dir, dest)) => {
            if stair_dir != dir {
                return Err(());
            }

            debug!(world.logger, "STAIR at {}: {:?}", pos, dest);

            Ok(dest)
        }
        _ => Err(())
    }
}

fn load_stair_dest(world: &mut EcsWorld, stair_pos: Point, next: StairDest) -> (EcsWorld, Point) {
    match next {
        StairDest::Generated(map_id, dest) => {
            debug!(world.logger, "Found stair leading to: {:?}", map_id);
            let map = world.get_map(map_id).unwrap();
            (map, dest)
        },
        StairDest::Ungenerated => {
            debug!(world.logger, "Failed to load map, generating...");
            let prev_id = world.map_id();
            let prev_seed = world.flags_mut().rng().next_u32();

            world.flags_mut().globals.max_map_id += 1;
            let next_id = world.flags().globals.max_map_id;

            let res = {
                let mut stairs_mut = world.cell_mut(&stair_pos).unwrap();

                generate_stair_dest(prev_id,
                                    next_id,
                                    prev_seed,
                                    stair_pos,
                                    stairs_mut)
            };
            debug!(world.logger, "new stairs: {:?}", world.cell_const(&stair_pos));
            res
        },
    }
}

fn generate_stair_dest(prev_id: MapId,
                       next_id: MapId,
                       seed: u32,
                       old_pos: Point,
                       stairs: &mut Cell)
                       -> (EcsWorld, Point) {
    let mut new_world = EcsWorld::from_prefab("prefab", seed, next_id);

    if let Some(CellFeature::Stairs(stair_dir, ref mut dest@StairDest::Ungenerated)) = stairs.feature {
        let new_stair_pos = new_world.find_stairs_in()
            .expect("Generated world has no stairs!");

        *dest = StairDest::Generated(next_id, new_stair_pos);

        new_world.place_stairs(stair_dir.reverse(),
                               new_stair_pos,
                               prev_id,
                               old_pos);

        (new_world, new_stair_pos)
    } else {
        panic!("Stairs should have already been found by now...");
    }
}


use renderer;
use glium::glutin;
use glium::glutin::{VirtualKeyCode, ElementState};

fn look(context: &mut GameContext) {
    renderer::with_mut(|rc| {
        rc.start_loop(|renderer| {
            let events = renderer.poll_events();
            if !events.is_empty() {
                for event in events {
                    match event {
                        glutin::Event::KeyboardInput(ElementState::Pressed, _, Some(code)) => {
                            println!("Key: {:?}", code);
                            {
                                {
                                    let world = &mut context.state.world;
                                    match code {
                                        VirtualKeyCode::Escape => return renderer::Action::Stop,
                                        VirtualKeyCode::Up => world.flags_mut().camera.y -= 1,
                                        VirtualKeyCode::Down => world.flags_mut().camera.y += 1,
                                        VirtualKeyCode::Left => world.flags_mut().camera.x -= 1,
                                        VirtualKeyCode::Right => world.flags_mut().camera.x += 1,
                                        _ => (),
                                    }
                                    if let Some(mob) = world.mob_at(world.flags().camera) {
                                        mes!(world, "You see here a {}.", a=entity::name(mob, world));
                                    }
                                }

                                renderer.update(context);
                            }
                        },
                        _ => (),
                    }
                    renderer.render();
                }
            } else {
                renderer.render();
            }

            renderer::Action::Continue
        });
    });
}
