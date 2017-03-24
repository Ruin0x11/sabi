use std::collections::HashMap;
use std::collections::hash_map;
use std::io;
use std::fmt;

use actor::Actor;
use action::Action;
use tile::*;
use point::Point;
use chunk::*;
use glyph::*;
use slog::Logger;
use log;
use uuid::Uuid;

#[derive(Copy, Clone)]
pub enum Walkability {
    WalkthroughMonsters,
    BlockingMonsters,
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
    actors: HashMap<Uuid, Actor>,
    actor_ids_by_pos: HashMap<WorldPosition, Uuid>,

    // NOTE: I'm not sure it makes sense for a player to be tied to an existing
    // world, but it works for now.
    player_id: Option<Uuid>,
    // NOTE: Also must keep track of following actors, to move them between
    // areas.

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
            player_id: None,
            logger: log::make_logger("world").unwrap(),
        }
    }

    pub fn generate(type_: WorldType, chunk_size: i32) -> World {
        let chunks = match type_ {
            WorldType::Instanced(dimensions) => World::generate_chunked(chunk_size, dimensions),
            _   => HashMap::new(),
        };

        let mut world = World::new_empty(type_, chunk_size);
        world.chunks = chunks;

        debug!(world.logger, "World created, no. of chunks: {}", world.chunks.len());
        assert!(world.chunks.len() > 0, "No chunks created!");
        for chunk_index in world.chunks.keys() {
            debug!(world.logger, "Index: {}", chunk_index);
        }

        world
    }

    fn generate_chunked(chunk_size: i32, dimensions: Point) -> HashMap<ChunkIndex, Chunk> {
        assert!(dimensions.x >= 0);
        assert!(dimensions.y >= 0);

        let mut chunks = HashMap::new();
        let debug = "abcdefg";

        // ceiling
        let w = (dimensions.x + chunk_size - 1) / chunk_size;
        let h = (dimensions.y + chunk_size - 1) / chunk_size;

        for i in 0..w {
            for j in 0..h {
                let index = (j + (i * h)) as usize;
                let tile = Tile {
                    type_: TileType::Wall,
                    glyph: Glyph::Debug(debug.chars().nth(index).unwrap_or('x')),
                    feature: None,
                };
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

    pub fn is_pos_in_bounds(&self, world_pos: WorldPosition) -> bool {
        let chunk_index = self.chunk_index_from_world_pos(world_pos);
        self.chunk(chunk_index).is_some()
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
        let tile_walkable = |world_pos| {
            match self.cell(world_pos) {
                Some(cell) => {
                    let res = cell.tile.can_pass_through();
                    debug!(self.logger, "Tile: {:?} res: {}", cell.tile, res);
                    res
                },
                None       => false,
            }
        };
        match walkability {
            Walkability::BlockingMonsters if
                self.actor_at(world_pos).is_some() => false,
            _                                      => tile_walkable(world_pos),
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

    fn chunk_result<F, R>(&self, world_pos: WorldPosition, func: &mut F, default: R) -> R
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

    fn chunk_result_mut<F, R>(&self, world_pos: WorldPosition, func: &mut F, default: R) -> R
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

    pub fn cell(&self, world_pos: WorldPosition) -> Option<&Cell> {
        let chunk_pos = self.chunk_pos_from_world_pos(world_pos);
        let chunk_opt = self.chunk_from_world_pos(world_pos);
        match chunk_opt {
            Some(chunk) => {
                Some(chunk.cell(chunk_pos))
            },
            None => None,
        }
    }

    pub fn cell_mut(&mut self, world_pos: WorldPosition) -> Option<&mut Cell> {
        let chunk_pos = self.chunk_pos_from_world_pos(world_pos);
        let chunk_opt = self.chunk_mut_from_world_pos(world_pos);
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

    /// Return an iterator over the currently loaded set of Actors in this
    /// world across all chunks.
    pub fn actors(&mut self) -> hash_map::Values<Uuid, Actor> {
        self.actors.values()
    }

    pub fn actor_at(&self, world_pos: WorldPosition) -> Option<&Actor> {
        match self.actor_ids_by_pos.get(&world_pos) {
            Some(uuid) => {
                assert!(self.actors.contains_key(uuid), "Coord -> id, id -> actor maps out of sync!");
                self.actors.get(uuid)
            }
            None       => None
        }
    }

    pub fn add_actor(&mut self, actor: Actor) {
        assert!(!self.actors.contains_key(&actor.get_uuid()), "Actor with same UUID already exists!");
        self.actors.insert(actor.get_uuid(), actor);
    }

    pub fn remove_actor(&mut self, uuid: Uuid) {
        let removed: bool = self.actors.remove(&uuid).is_some();
        assert!(removed, "Tried removing nonexistent actor from world!");
    }

    pub fn player_id(&self) -> Uuid {
        self.player_id.unwrap()
    }

    pub fn set_player_id(&mut self, uuid: Uuid) {
        self.player_id = Some(uuid);
    }

    fn pre_tick(&mut self) {

    }

    pub fn run_action(&mut self, action: Action, uuid: Uuid) {
        self.pre_tick();
        let mut actor = self.actors.remove(&uuid).unwrap();

        actor.run_action(action, self);

        self.actors.insert(uuid, actor);
        self.post_tick();
    }

    fn post_tick(&mut self) {

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
    fn test_is_pos_in_bounds() {
        let world = World::generate(WorldType::Instanced(Point::new(1, 1)), 16);
        assert_eq!(world.is_pos_in_bounds(Point::new(0, 0)), true);
        for i in -1..1 {
            for j in -1..1 {
                if i != 0 && j != 0 {
                    let pos = Point::new(i, j);
                    let index = world.chunk_index_from_world_pos(pos);
                    assert_eq!(world.is_pos_in_bounds(pos), false, "pos: {} index: {}", pos, index);
                }
            }
        }

        let world = World::generate(WorldType::Instanced(Point::new(32, 32)), 16);
        assert_eq!(world.is_pos_in_bounds(Point::new(0, 0)), true);
        assert_eq!(world.is_pos_in_bounds(Point::new(17, 17)), true);
        assert_eq!(world.is_pos_in_bounds(Point::new(32, 17)), false);
        assert_eq!(world.is_pos_in_bounds(Point::new(-1, -1)), false);
    }
}
