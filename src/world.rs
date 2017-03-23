use std::collections::HashMap;
use std::io;
use std::fmt;

use tile::*;
use point::Point;
use chunk::*;
use glyph::*;
use slog::Logger;
use log;

// Because a world position and chunk index are different quantities, newtype to
// enforce corrent usage
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

pub type WorldPosition = Point;

pub enum WorldType {
    Instanced(Point),
    Overworld
}

/// Describes a collection of Chunks put together to form a complete playing
/// field. This is the interface to the world that living beings can interact
/// with.
pub struct World {
    chunk_size: i32,
    chunks: HashMap<ChunkIndex, Chunk>,
    type_: WorldType,
    pub logger: Logger,
}

impl World {
    pub fn new(type_: WorldType, chunk_size: i32) -> Self {
        World {
            chunk_size: chunk_size,
            chunks: HashMap::new(),
            type_: type_,
            logger: log::make_logger("world").unwrap(),
        }
    }

    pub fn generate(chunk_size: i32, type_: WorldType) -> World {
        let chunks = match type_ {
            WorldType::Instanced(dimensions) => World::generate_chunked(chunk_size, dimensions),
            _   => HashMap::new(),
        };

        let mut world = World::new(type_, chunk_size);
        world.chunks = chunks;

        debug!(world.logger, "World created, no. of chunks: {}", world.chunks.len());

        world
    }

    fn generate_chunked(chunk_size: i32, dimensions: Point) -> HashMap<ChunkIndex, Chunk> {
        assert!(dimensions.x >= 0);
        assert!(dimensions.y >= 0);

        let mut chunks = HashMap::new();
        let debug = "abcdefg";

        let w = dimensions.x / chunk_size;
        let h = dimensions.y / chunk_size;
        for i in 0..w {
            for j in 0..h {
                let index = (j + (i * h)) as usize;
                let tile = Tile {
                    type_: TileType::Floor,
                    glyph: Glyph::Debug(debug.chars().nth(index).unwrap_or('x')),
                    feature: None,
                };
                // TODO: Shave off bounds
                chunks.insert(ChunkIndex::new(i, j), Chunk::generate_basic(chunk_size, tile));
            }
        }
        chunks
    }

    fn chunk_info_from_world_pos(&self, pos: WorldPosition) -> (ChunkIndex, Point) {
        (self.chunk_index_from_world_pos(pos), self.world_pos_to_chunk_pos(pos))
    }

    fn chunk_index_from_world_pos(&self, pos: WorldPosition) -> ChunkIndex {
        let chunk_size_i = self.chunk_size as i32;
        let conv = |i: i32| i / chunk_size_i;

        ChunkIndex::new(conv(pos.x), conv(pos.y))
    }

    pub fn chunk_from_world_pos(&self, pos: WorldPosition) -> Option<&Chunk> {
        let index = self.chunk_index_from_world_pos(pos);
        self.get_chunk(index)
    }

    pub fn chunk_mut_from_world_pos(&mut self, pos: WorldPosition) -> Option<&mut Chunk> {
        let index = self.chunk_index_from_world_pos(pos);
        self.get_chunk_mut(index)
    }

    pub fn get_chunk(&self, index: ChunkIndex) -> Option<&Chunk> {
        self.chunks.get(&index)
    }

    pub fn get_chunk_mut(&mut self, index: ChunkIndex) -> Option<&mut Chunk> {
        self.chunks.get_mut(&index)
    }

    fn world_pos_to_chunk_pos(&self, pos: WorldPosition) -> Point {
        let chunk_size_i = self.chunk_size as i32;
        let conv = |i: i32| {
            let i_new = i % chunk_size_i;
            if i_new < 0 {
                chunk_size_i + i_new
            } else {
                i_new
            }
        };

        Point::new(conv(pos.x), conv(pos.y))
    }

    pub fn is_pos_valid(&self, world_pos: WorldPosition) -> bool {
        let chunk_index = self.chunk_index_from_world_pos(world_pos);
        self.get_chunk(chunk_index).is_some()
    }

    pub fn cell(&self, world_pos: WorldPosition) -> Option<&Cell> {
        let chunk_pos = self.world_pos_to_chunk_pos(world_pos);
        let chunk_o = self.chunk_from_world_pos(world_pos);
        match chunk_o {
            Some(chunk) => {
                Some(chunk.cell(chunk_pos))
            },
            None => None,
        }
    }

    pub fn cell_mut(&mut self, world_pos: WorldPosition) -> Option<&mut Cell> {
        let chunk_pos = self.world_pos_to_chunk_pos(world_pos);
        let chunk_o = self.chunk_mut_from_world_pos(world_pos);
        match chunk_o {
            Some(chunk) => {
                Some(chunk.cell_mut(chunk_pos))
            }
            None => None,
        }
    }

    // IMPLEMENT
    /// Return an iterator over the currently loaded set of Actors in this
    /// world across all chunks.
    #[cfg(never)]
    pub fn actors() {

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


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_world_pos_to_chunk_pos() {
        let chunk_size = 128;
        let world = World::new(WorldType::Overworld, chunk_size);
        assert_eq!(
            world.world_pos_to_chunk_pos(Point::new(0, 0)),
            Point::new(0, 0)
        );
        assert_eq!(
            world.world_pos_to_chunk_pos(Point::new(1, 1)),
            Point::new(1, 1)
        );
        assert_eq!(
            world.world_pos_to_chunk_pos(Point::new(chunk_size, chunk_size)),
            Point::new(0, 0)
        );
        assert_eq!(
            world.world_pos_to_chunk_pos(Point::new(chunk_size * 2 + 64, chunk_size * 2 + 32)),
            Point::new(64, 32)
        );
        assert_eq!(
            world.world_pos_to_chunk_pos(Point::new(-chunk_size, -chunk_size)),
            Point::new(0, 0)
        );
        assert_eq!(
            world.world_pos_to_chunk_pos(Point::new(-chunk_size * 2 + 64, -chunk_size * 2 + 32)),
            Point::new(64, 32)
        );
    }

    #[test]
    fn test_chunk_index_from_world_pos() {
        let chunk_size = 128;
        let world = World::new(WorldType::Overworld, chunk_size);
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
            world.chunk_index_from_world_pos(Point::new(chunk_size * 2 + 64, chunk_size * 3 + 32)),
            ChunkIndex::new(2, 3)
        );
        assert_eq!(
            world.chunk_index_from_world_pos(Point::new(-chunk_size, -chunk_size)),
            ChunkIndex::new(-1, -1)
        );
        assert_eq!(
            world.chunk_index_from_world_pos(Point::new(-chunk_size * 3 + 64, -chunk_size * 4 + 32)),
            ChunkIndex::new(-2, -3)
        );
    }
}
