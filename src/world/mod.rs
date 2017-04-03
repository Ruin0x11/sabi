mod actors;
mod iterators;
mod message;
mod turn_order;

use std::cell::RefCell;
use std::collections::{VecDeque, HashMap, HashSet, hash_map};
use std::fmt;

use actor::{Actor, ActorId};
use action::Action;
use drawcalls::*;
use event::Event;
use tile::*;
use point::Point;
use chunk::*;
use glyph::*;
use slog::Logger;
use log;

use self::turn_order::TurnOrder;
use self::message::Messages;

pub type WorldIter = Iterator<Item=WorldPosition>;

#[derive(Eq, PartialEq, Copy, Clone)]
pub enum Walkability {
    MonstersWalkable,
    MonstersBlocking,
}

pub type WorldPosition = Point;

pub enum WorldType {
    Instanced(Point),
    Overworld,
    Nothing
}

/// Describes a collection of Chunks put together to form a complete playing
/// field. This is the interface to the world that living beings can interact
/// with.
pub struct World {
    chunk_size: i32,
    chunks: HashMap<ChunkIndex, Chunk>,
    type_: WorldType,

    // NOTE: could also implement by putting each in its own Chunk
    actors: HashMap<ActorId, Actor>,
    actor_ids_by_pos: HashMap<WorldPosition, ActorId>,
    // Actors that were killed during the current actor's turn, by events, etc.
    killed_actors: HashMap<ActorId, Actor>,

    // NOTE: I'm not sure it makes sense for a player to be tied to an existing
    // world, but it works for now.
    player_id: Option<ActorId>,
    // NOTE: Also must keep track of following actors, to move them between
    // areas.

    turn_order: TurnOrder,
    pub draw_calls: DrawCalls,
    messages: RefCell<Messages>,
    pub events: Vec<Event>,
    pub logger: Logger,
}

impl World {
    pub fn new_empty(type_: WorldType, chunk_size: i32) -> Self {
        World {
            chunk_size: chunk_size,
            chunks: HashMap::new(),
            type_: type_,
            actors: HashMap::new(),
            actor_ids_by_pos: HashMap::new(),
            killed_actors: HashMap::new(),
            player_id: None,
            turn_order: TurnOrder::new(),
            draw_calls: DrawCalls::new(),
            events: Vec::new(),
            messages: RefCell::new(Messages::new()),
            logger: log::make_logger("world").unwrap(),
        }
    }

    pub fn generate(type_: WorldType, chunk_size: i32, tile: Tile) -> World {
        let chunks = match type_ {
            WorldType::Instanced(dimensions) => World::generate_chunked(chunk_size, dimensions, tile),
            _   => HashMap::new(),
        };

        let mut world = World::new_empty(type_, chunk_size);
        world.chunks = chunks;

        assert!(world.chunks.len() > 0, "No chunks created!");
        debug!(world.logger, "World created, no. of chunks: {}", world.chunks.len());
        for chunk_index in world.chunks.keys() {
            debug!(world.logger, "Index: {}", chunk_index);
        }

        world
    }

    fn generate_chunked(chunk_size: i32, dimensions: Point, tile: Tile) -> HashMap<ChunkIndex, Chunk> {
        assert!(dimensions.x >= 0);
        assert!(dimensions.y >= 0);

        let mut chunks = HashMap::new();
        let debug = "abcdefg";

        // ceiling
        let columns = (dimensions.x + chunk_size - 1) / chunk_size;
        let rows = (dimensions.y + chunk_size - 1) / chunk_size;

        for i in 0..columns {
            for j in 0..rows {
                let index = (j + (i * rows)) as usize;
                // TODO: Shave off bounds
                chunks.insert(ChunkIndex::new(i, j), Chunk::generate_basic(chunk_size, tile));
            }
        }
        chunks
    }

    fn chunk_index_from_world_pos(&self, pos: WorldPosition) -> ChunkIndex {
        let conv = |i: i32| {
            if i < 0 {
                // [-1, -chunk_size] = -1
                ((i + 1) / self.chunk_size) - 1
            } else {
                // [0, chunk_size-1] = 0
                i / self.chunk_size
            }
        };

        ChunkIndex::new(conv(pos.x), conv(pos.y))
    }

    fn chunk_pos_from_world_pos(&self, pos: WorldPosition) -> Point {
        let conv = |i: i32| {
            let i_new = i % self.chunk_size;
            if i_new < 0 {
                self.chunk_size + i_new
            } else {
                i_new
            }
        };

        Point::new(conv(pos.x), conv(pos.y))
    }

    pub fn chunk_from_world_pos(&self, pos: WorldPosition) -> Option<&Chunk> {
        let index = self.chunk_index_from_world_pos(pos);
        self.chunk(index)
    }

    pub fn chunk_mut_from_world_pos(&mut self, pos: WorldPosition) -> Option<&mut Chunk> {
        let index = self.chunk_index_from_world_pos(pos);
        self.chunk_mut(index)
    }

    pub fn chunk(&self, index: ChunkIndex) -> Option<&Chunk> {
        self.chunks.get(&index)
    }

    pub fn chunk_mut(&mut self, index: ChunkIndex) -> Option<&mut Chunk> {
        self.chunks.get_mut(&index)
    }

    pub fn pos_valid(&self, world_pos: &WorldPosition) -> bool {
        let chunk_index = self.chunk_index_from_world_pos(*world_pos);
        let is_in_chunk = self.chunk(chunk_index).is_some();
        match self.type_ {
            WorldType::Instanced(size) => {
                let is_in_boundaries = *world_pos < size;
                // debug!(self.logger, "pos: {} size: {}", world_pos, size);
                // debug!(self.logger, "in chunk, boundaries: {} {}", is_in_chunk, is_in_boundaries);
                is_in_chunk && is_in_boundaries
            }
            _ => is_in_chunk
        }
    }

    pub fn see_tile(&mut self, world_pos: WorldPosition) {
        let chunk_pos = self.chunk_pos_from_world_pos(world_pos);
        let chunk_opt = self.chunk_mut_from_world_pos(world_pos);
        if let Some(chunk) = chunk_opt {
            chunk.see_tile(chunk_pos);
        }
    }

    pub fn is_walkable(&self, world_pos: WorldPosition,
                       walkability: Walkability) -> bool {
        let walkable = match walkability {
            Walkability::MonstersWalkable => true,
            Walkability::MonstersBlocking => self.actor_at(world_pos).is_none(),
        };

        match self.cell(&world_pos) {
            Some(cell) => {
                let passable = cell.tile.can_pass_through();
                let in_bounds = self.pos_valid(&world_pos);
                // debug!(self.logger, "Cell: {:?}", cell);
                // debug!(self.logger, "passable, bounds, walkable: {} {} {}",
                // passable, in_bounds, walkable);
                passable && in_bounds && walkable
            },
            None       => false,
        }
    }

    pub fn is_visible(&self, world_pos: WorldPosition) -> bool {
        let chunk_pos = self.chunk_pos_from_world_pos(world_pos);
        let chunk_opt = self.chunk_from_world_pos(world_pos);
        match chunk_opt {
            Some(chunk) => {
                chunk.is_seen(chunk_pos)
            },
            None => false,
        }
    }

    // FIXME: It complains about adding lifetimes.
    fn chunk_result<'a, F, R>(&'a self, world_pos: WorldPosition, func: &mut F, default: R) -> R
        where F: FnMut(&Chunk, ChunkPosition) -> R {
        let chunk_pos = self.chunk_pos_from_world_pos(world_pos);
        let chunk_opt = self.chunk_from_world_pos(world_pos);
        match chunk_opt {
            Some(chunk) => {
                func(chunk, chunk_pos)
            },
            None => default,
        }
    }

    fn chunk_result_mut<'a, F, R>(&'a self, world_pos: WorldPosition, func: &mut F, default: R) -> R
        where F: FnMut(&Chunk, ChunkPosition) -> R {
        let chunk_pos = self.chunk_pos_from_world_pos(world_pos);
        let chunk_opt = self.chunk_from_world_pos(world_pos);
        match chunk_opt {
            Some(chunk) => {
                func(chunk, chunk_pos)
            },
            None => default,
        }
    }

    pub fn cell(&self, world_pos: &WorldPosition) -> Option<&Cell> {
        let chunk_pos = self.chunk_pos_from_world_pos(*world_pos);
        let chunk_opt = self.chunk_from_world_pos(*world_pos);
        match chunk_opt {
            Some(chunk) => {
                Some(chunk.cell(chunk_pos))
            },
            None => None,
        }
    }

    pub fn cell_mut(&mut self, world_pos: &WorldPosition) -> Option<&mut Cell> {
        let chunk_pos = self.chunk_pos_from_world_pos(*world_pos);
        let chunk_opt = self.chunk_mut_from_world_pos(*world_pos);
        match chunk_opt {
            Some(chunk) => {
                Some(chunk.cell_mut(chunk_pos))
            }
            None => None,
        }
    }

    /// Return an iterator over `Cell` that covers a rectangular shape
    /// specified by the top-left (inclusive) point and the dimensions
    /// (width, height) of the rectangle.
    ///
    /// The iteration order is not specified.
    pub fn with_cells<F>(&mut self, top_left: WorldPosition, dimensions: Point, mut callback: F)
        where F: FnMut(Point, &Cell)
    {
        assert!(dimensions.x >= 0);
        assert!(dimensions.y >= 0);

        let mut chunk_index = self.chunk_index_from_world_pos(top_left);
        let mut world_pos = Chunk::world_position_from_index(chunk_index, self.chunk_size);
        let bottom_right = top_left + dimensions;
        let starter_chunk_x = world_pos.x;

        while world_pos.y < bottom_right.y {
            while world_pos.x < bottom_right.x {
                {
                    chunk_index = self.chunk_index_from_world_pos(world_pos);
                    let chunk_opt = self.chunk_from_world_pos(world_pos);
                    if let Some(chunk) = chunk_opt {
                        for (chunk_pos, cell) in chunk.iter() {
                            let cell_world_pos = chunk.world_position(chunk_index, chunk_pos);
                            if cell_world_pos >= top_left && cell_world_pos < bottom_right {
                                callback(cell_world_pos, cell);
                            }
                        }
                    }
                }
                world_pos.x += self.chunk_size;
            }
            world_pos.y += self.chunk_size;
            world_pos.x = starter_chunk_x;
        }

    }
}

// Because a world position and chunk index are different quantities, newtype to
// enforce correct usage
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub struct ChunkIndex(pub Point);

impl ChunkIndex {
    fn new(x: i32, y: i32) -> Self {
        ChunkIndex(Point::new(x, y))
    }
}

impl fmt::Display for ChunkIndex {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tile::{self, FLOOR};
    use direction::Direction;
    use logic;
    use testbed::make_grid_from_str;

    fn world_from_str(text: &str) -> World {
        let callback = |pt: &Point, c: char, world: &mut World| {
            if c == '@' {
                let actor = Actor::from_archetype(pt.x, pt.y, "putit");
                world.add_actor(actor);
            }

            let cell_kind = match c {
                '.' => tile::FLOOR,
                '#' => tile::WALL,
                _   => unreachable!(),
            };
            world.set_tile(pt.clone(), cell_kind);
        };
        let make = |dim: Point| World::generate(WorldType::Instanced(dim), 64, tile::FLOOR);
        make_grid_from_str(text, make, callback)
    }

    fn get_world(size: Point) -> World {
        World::generate(WorldType::Instanced(size), 16, tile::FLOOR)
    }

    #[test]
    fn test_chunk_pos_from_world_pos() {
        let chunk_size = 128;
        let world = World::new_empty(WorldType::Overworld, chunk_size);
        assert_eq!(
            world.chunk_pos_from_world_pos(Point::new(0, 0)),
            Point::new(0, 0)
        );
        assert_eq!(
            world.chunk_pos_from_world_pos(Point::new(1, 1)),
            Point::new(1, 1)
        );
        assert_eq!(
            world.chunk_pos_from_world_pos(Point::new(chunk_size, chunk_size)),
            Point::new(0, 0)
        );
        assert_eq!(
            world.chunk_pos_from_world_pos(Point::new(chunk_size * 2 + 64, chunk_size * 2 + 32)),
            Point::new(64, 32)
        );
        assert_eq!(
            world.chunk_pos_from_world_pos(Point::new(-chunk_size, -chunk_size)),
            Point::new(0, 0)
        );
        assert_eq!(
            world.chunk_pos_from_world_pos(Point::new(-chunk_size * 2 + 64, -chunk_size * 2 + 32)),
            Point::new(64, 32)
        );
    }

    #[test]
    fn test_chunk_index_from_world_pos() {
        let chunk_size = 128;
        let world = World::new_empty(WorldType::Overworld, chunk_size);
        assert_eq!(
            world.chunk_index_from_world_pos(Point::new(0, 0)),
            ChunkIndex::new(0, 0)
        );
        assert_eq!(
            world.chunk_index_from_world_pos(Point::new(1, 1)),
            ChunkIndex::new(0, 0)
        );
        assert_eq!(
            world.chunk_index_from_world_pos(Point::new(chunk_size - 1, chunk_size - 1)),
            ChunkIndex::new(0, 0)
        );
        assert_eq!(
            world.chunk_index_from_world_pos(Point::new(chunk_size, chunk_size)),
            ChunkIndex::new(1, 1)
        );
        assert_eq!(
            world.chunk_index_from_world_pos(Point::new(chunk_size * 2 + (chunk_size / 2),
                                                        chunk_size * 3 + (chunk_size / 2))),
            ChunkIndex::new(2, 3)
        );
        assert_eq!(
            world.chunk_index_from_world_pos(Point::new(-chunk_size, -chunk_size)),
            ChunkIndex::new(-1, -1)
        );
        assert_eq!(
            world.chunk_index_from_world_pos(Point::new(-chunk_size + (chunk_size / 2),
                                                        -chunk_size + (chunk_size / 2))),
            ChunkIndex::new(-1, -1)
        );
        assert_eq!(
            world.chunk_index_from_world_pos(Point::new(-chunk_size * 3 + (chunk_size / 2),
                                                        -chunk_size * 4 + (chunk_size / 2))),
            ChunkIndex::new(-3, -4)
        );
    }

    #[test]
    fn test_pos_valid() {
        let world = get_world(Point::new(1, 1));
        assert_eq!(world.pos_valid(&Point::new(0, 0)), true);
        for i in -1..1 {
            for j in -1..1 {
                if i != 0 && j != 0 {
                    let pos = Point::new(i, j);
                    let index = world.chunk_index_from_world_pos(pos);
                    assert_eq!(world.pos_valid(&pos), false, "pos: {} index: {}", pos, index);
                }
            }
        }

        let world = get_world(Point::new(32, 32));
        assert_eq!(world.pos_valid(&Point::new(0, 0)), true);
        assert_eq!(world.pos_valid(&Point::new(17, 17)), true);
        assert_eq!(world.pos_valid(&Point::new(32, 17)), false);
        assert_eq!(world.pos_valid(&Point::new(-1, -1)), false);
    }

    #[test]
    fn test_actor_at() {
        let mut world = get_world(Point::new(2, 2));
        let pos = Point::new(0, 0);
        assert!(world.actor_at(pos).is_none());

        let actor = Actor::new(0, 0, Glyph::Player);
        world.add_actor(actor);
        assert!(world.actor_at(pos).is_some());

        let next_actor = world.next_actor().unwrap();
        logic::run_action(&mut world, &next_actor, Action::Move(Direction::SE));
        assert!(world.actor_at(pos).is_none(), "{:?}", world.actor_ids_by_pos);
        assert!(world.actor_at(Point::new(1,1)).is_some());

        world.remove_actor(&next_actor);
        assert!(world.actor_at(Point::new(1,1)).is_none());
    }

    #[test]
    fn test_is_walkable() {
        let world = world_from_str(".");

        assert_eq!(world.is_walkable(Point::new(-1, -1), Walkability::MonstersWalkable), false);
        assert_eq!(world.is_walkable(Point::new(1, 1), Walkability::MonstersWalkable), false);

        let world = world_from_str("
@.
..");
        assert_eq!(world.is_walkable(Point::new(0, 0), Walkability::MonstersBlocking), false);
        assert_eq!(world.is_walkable(Point::new(0, 0), Walkability::MonstersWalkable), true);
    }
}
