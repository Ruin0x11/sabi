use std::collections::HashMap;

use tile::*;
use point::Point;
use chunk::*;
use glyph::*;
use slog::Logger;
use log;

pub type ChunkIndex = Point;

pub type WorldPosition = Point;

pub enum WorldType {
    Instanced(Point),
    Overworld
}

/// Describes a collection of Chunks put together to form a complete playing
/// field. This is the interface to the world that living beings can interact
/// with. 
pub struct World {
    chunk_size: u32,
    chunks: HashMap<ChunkIndex, Chunk>,
    type_: WorldType,
    pub logger: Logger,
}

impl World {
    pub fn new(type_: WorldType, chunk_size: u32) -> World {
        World {
            chunk_size: chunk_size,
            chunks: HashMap::new(),
            type_: type_,
            logger: log::make_logger("world").unwrap(),
        }
    }

    pub fn generate(chunk_size: u32, type_: WorldType) -> World {
        let chunks = match type_ {
            WorldType::Instanced(dimensions) => World::generate_chunked(chunk_size, dimensions),
            _   => HashMap::new(),
        };

        let mut world = World::new(type_, chunk_size);
        world.chunks = chunks;

        debug!(world.logger, "World created, no. of chunks: {}", world.chunks.len());

        world
    }

    fn generate_chunked(chunk_size: u32, dimensions: Point) -> HashMap<ChunkIndex, Chunk> {
        assert!(dimensions.x >= 0);
        assert!(dimensions.y >= 0);

        let mut chunks = HashMap::new();
        let chunk_size_i = chunk_size as i32;
        let debug = "abcdefg";

        let w = dimensions.x / chunk_size_i;
        let h = dimensions.y / chunk_size_i;
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

    fn world_pos_to_chunk_info(&self, pos: WorldPosition) -> (ChunkIndex, Point) {
        (self.world_pos_to_chunk_index(pos), self.world_pos_to_chunk_pos(pos))
    }

    fn world_pos_to_chunk_index(&self, pos: WorldPosition) -> ChunkIndex {
        let chunk_size_i = self.chunk_size as i32;
        let conv = |i: i32| i / chunk_size_i;

        ChunkIndex::new(conv(pos.x), conv(pos.y))
    }

    pub fn get_chunk_from_world_pos(&self, pos: WorldPosition) -> Option<&Chunk> {
        let index = self.world_pos_to_chunk_index(pos);
        debug!(self.logger, "Chunk index: {} pos: {}", index, pos);
        self.get_chunk(index)
    }

    pub fn get_chunk_mut_from_world_pos(&mut self, pos: WorldPosition) -> Option<&mut Chunk> {
        let index = self.world_pos_to_chunk_index(pos);
        debug!(self.logger, "Chunk index: {} pos: {}", index, pos);
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
        let chunk_index = self.world_pos_to_chunk_index(world_pos);
        self.get_chunk(chunk_index).is_some()
    }

    pub fn cell(&self, world_pos: WorldPosition) -> Option<&Cell> {
        assert!(self.is_pos_valid(world_pos), "invalid pos {}", &world_pos);
        let chunk_pos = self.world_pos_to_chunk_pos(world_pos);
        let chunk_o = self.get_chunk_from_world_pos(world_pos);
        match chunk_o {
            Some(chunk) => {
                Some(chunk.cell(chunk_pos))
            },
            None => None,
        }
    }

    pub fn cell_mut(&mut self, world_pos: WorldPosition) -> Option<&mut Cell> {
        assert!(self.is_pos_valid(world_pos), "invalid pos {}", &world_pos);
        let chunk_pos = self.world_pos_to_chunk_pos(world_pos);
        let chunk_o = self.get_chunk_mut_from_world_pos(world_pos);
        match chunk_o {
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

        // IMPLEMENT
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_world_pos_to_chunk_pos() {
        let chunk_size = 128;
        let world = World::new(WorldType::Overworld, chunk_size as u32);
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
    fn test_world_pos_to_chunk_index() {
        let chunk_size = 128;
        let world = World::new(WorldType::Overworld, chunk_size as u32);
        assert_eq!(
            world.world_pos_to_chunk_index(Point::new(0, 0)),
            ChunkIndex::new(0, 0)
        );
        assert_eq!(
            world.world_pos_to_chunk_index(Point::new(1, 1)),
            ChunkIndex::new(0, 0)
        );
        assert_eq!(
            world.world_pos_to_chunk_index(Point::new(chunk_size - 1, chunk_size - 1)),
            ChunkIndex::new(0, 0)
        );
        assert_eq!(
            world.world_pos_to_chunk_index(Point::new(chunk_size, chunk_size)),
            ChunkIndex::new(1, 1)
        );
        assert_eq!(
            world.world_pos_to_chunk_index(Point::new(chunk_size * 2 + 64, chunk_size * 3 + 32)),
            ChunkIndex::new(2, 3)
        );
        assert_eq!(
            world.world_pos_to_chunk_index(Point::new(-chunk_size, -chunk_size)),
            ChunkIndex::new(-1, -1)
        );
        assert_eq!(
            world.world_pos_to_chunk_index(Point::new(-chunk_size * 3 + 64, -chunk_size * 4 + 32)),
            ChunkIndex::new(-2, -3)
        );
    }
}
