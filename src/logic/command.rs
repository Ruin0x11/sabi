use rand::{self, Rng};

use point::Direction;
use engine::keys::{Key, KeyCode};
use graphics::cell::{Cell, CellFeature, StairDest, StairDir};
use world::{Bounds, EcsWorld};
use point::Point;
use chunk::ChunkIndex;
use chunk::generator::ChunkType;
use infinigen::ChunkedWorld;
use world::MapId;
use world::traits::*;

pub type CommandResult = Result<(), ()>;

pub enum Command {
    Move(Direction),
    UseStairs(StairDir),
    Wait,
    Quit,
}

pub fn try_use_stairs(dir: StairDir, world: &mut EcsWorld) -> CommandResult {
    let player = match world.player() {
        Some(p) => p,
        None    => return Err(()),
    };

    let pos = match world.position(player) {
        Some(p) => p,
        None    => return Err(()),
    };

    let next = match find_stair_dest(world, pos, dir) {
        Ok(n)  => n,
        Err(_) => return Err(()),
    };

    let (true_next, dest) = load_stair_dest(world, pos, next);
    world.move_to_map(true_next, dest).unwrap();

    debug!(world.logger, "map id: {:?}", world.map_id());
    Ok(())
}

fn find_stair_dest(world: &EcsWorld, pos: Point, dir: StairDir) -> Result<StairDest, ()> {
    let cell = match world.terrain().cell(&pos) {
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
                let mut stairs_mut = world.terrain_mut().cell_mut(&stair_pos).unwrap();

                generate_stair_dest(prev_id,
                                    next_id,
                                    prev_seed,
                                    stair_pos,
                                    stairs_mut)
            };
            debug!(world.logger, "new stairs: {:?}", world.terrain().cell(&stair_pos));
            res
        },
    }
}

fn generate_stair_dest(prev_id: MapId, next_id: MapId, seed: u32, old_pos: Point, stairs: &mut Cell) -> (EcsWorld, Point) {
    // TODO: This should be replaced with the "make from prefab" function
    let mut new_world = EcsWorld::new(Bounds::Bounded(64, 64), ChunkType::Lua, seed);

    // TODO: make better. Too many traits to import also.

    if let Some(CellFeature::Stairs(stair_dir, ref mut dest@StairDest::Ungenerated)) = stairs.feature {
        let dest_id = next_id;
        let dest_pos = Point::new(0, 0);
        *dest = StairDest::Generated(dest_id, dest_pos);

        let new_stair_pos = Point::new(3, 3);


        // TODO: Make a framework for temporarily loading chunks like this.
        // This is why. If one does not set the correct map_id before generating
        // chunks in the new world, they are not saved to the correct directory.
        new_world.set_map_id(dest_id);

        new_world.load_chunk(&ChunkIndex::from(new_stair_pos)).unwrap();
        new_world.terrain_mut()
            .place_stairs(stair_dir.reverse(),
                          new_stair_pos,
                          prev_id,
                          old_pos);
        new_world.unload_chunk(&ChunkIndex::from(new_stair_pos)).unwrap();
        // but then the maximum map id in the new world has changed

        (new_world, dest_pos)
    } else {
        panic!("Stairs should have already been found by now...");
    }
}

impl Command {
    pub fn from_key(key: Key) -> Command {
        match key {
            Key { code: KeyCode::Esc,         .. } => Command::Quit,
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

            Key { code: KeyCode::GreaterThan, .. } => Command::UseStairs(StairDir::Ascending),
            Key { code: KeyCode::LessThan,    .. } => Command::UseStairs(StairDir::Descending),

            _                                  => Command::Wait
        }
    }
}
