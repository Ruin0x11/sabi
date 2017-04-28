use point::Direction;
use engine::keys::{Key, KeyCode};
use graphics::cell::{CellFeature, StairDir};
use world::{Bounds, EcsWorld};
use point::Point;
use chunk::generator::ChunkType;
use infinigen::ChunkedWorld;
use world::traits::*;

pub type CommandResult = Result<(), ()>;

pub enum Command {
    Move(Direction),
    UseStairs(StairDir),
    Wait,
    Quit,
}

pub fn try_use_stairs(dir: StairDir, world: &mut EcsWorld) -> CommandResult {
    let player = match world.flags().player {
        Some(p) => p,
        None    => return Err(()),
    };

    let pos = match world.position(player) {
        Some(p) => p,
        None    => return Err(()),
    };

    let next: Option<EcsWorld> = {
        let mut cell = match world.terrain().cell(&pos) {
            Some(c) => c,
            None    => return Err(()),
        };

        match cell.feature {
            Some(CellFeature::Stairs(stair_dir, map_id, pos)) => {
                if stair_dir != dir {
                    return Err(())
                }
                debug!(world.logger, "Found stair, id: {:?}", map_id);

                world.get_map(map_id)
            }
            _ => return Err(())
        }
    };

    let (mut true_next, dest): (EcsWorld, Point) = match next {
        Some(map) => (map, world.terrain().cell(&pos).unwrap().stair_dest().unwrap()),
        None      => {
            let mut new_world = EcsWorld::new(Bounds::Bounded(32, 32), ChunkType::Blank, world.flags().seed);
            // TODO: make better. Too many traits to import also.

            let mut stairs = world.terrain_mut().cell_mut(&pos).unwrap();
            if let Some(CellFeature::Stairs(_, stairs_map_id, mut stair_pos)) = stairs.feature {
                stair_pos = Some(Point::new(0, 0));

                // TODO: shouldn't have to set manually.
                new_world.flags_mut().map_id = stairs_map_id;
                new_world.terrain_mut().set_id(stairs_map_id);

                (new_world, stair_pos.unwrap())
            } else {
                return Err(())
            }
        },
    };

    world.move_to_map(true_next).unwrap();

    debug!(world.logger, "map id: {:?}", world.flags().map_id);
    Ok(())
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
