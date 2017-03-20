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
/// field. 
pub struct World {
    chunk_size: u32,
    chunks: HashMap<ChunkIndex, Chunk>,
    type_: WorldType,
    pub logger: Logger,
}

impl World {
    pub fn new(chunk_size: u32) -> World {
        World {
            chunk_size: chunk_size,
            chunks: HashMap::new(),
            type_: WorldType::Overworld,
            logger: log::make_logger("world").unwrap(),
        }
    }

    pub fn generate(chunk_size: u32, type_: WorldType) -> World {
        let type_ = WorldType::Instanced(Point::new(64, 64));
        let tile = Tile {
            type_: TileType::Floor,
            glyph: Glyph::Floor,
            feature: None,
        };

        let chunks = match type_ {
            WorldType::Instanced(point) => {
                let mut v = HashMap::new();
                v.insert(ChunkIndex::new(0, 0), Chunk::generate_basic(chunk_size, tile));
                v
            },
            _   => HashMap::new(),
        };

        let mut world = World::new(chunk_size);
        world.chunks = chunks;

        let chunk_size_i = chunk_size as i32;
        let tile = Tile {
            type_: TileType::Wall,
            glyph: Glyph::Wall,
            feature: None,
        };

        world.draw_rect(WorldPosition::new(0, 0),
                        WorldPosition::new(24, 24),
                        tile);
        world
    }

    fn world_pos_to_chunk_info(&self, pos: WorldPosition) -> (ChunkIndex, Point) {
        (self.world_pos_to_chunk_index(pos), self.world_pos_to_chunk_pos(pos))
    }

    fn world_pos_to_chunk_index(&self, pos: WorldPosition) -> ChunkIndex {
        let chunk_size_i = self.chunk_size as i32;
        let conv = |i: i32| i / (chunk_size_i - 1);

        ChunkIndex::new(conv(pos.x), conv(pos.y))
    }

    pub fn get_chunk(&self, pos: WorldPosition) -> Option<&Chunk> {
        let pos = self.world_pos_to_chunk_index(pos);
        self.chunks.get(&pos)
    }

    pub fn get_chunk_mut(&mut self, pos: WorldPosition) -> Option<&mut Chunk> {
        let pos = self.world_pos_to_chunk_index(pos);
        self.chunks.get_mut(&pos)
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

    pub fn cell(&mut self, world_pos: WorldPosition) -> Option<&Cell> {
        assert!(self.is_pos_valid(world_pos), "invalid pos {}", &world_pos);
        let (chunk_index, chunk_pos) = self.world_pos_to_chunk_info(world_pos);
        let chunk_o = self.get_chunk(chunk_index);
        match chunk_o {
            Some(chunk) => {
                Some(chunk.cell(chunk_pos))
            },
            None => None,
        }
    }
    

    pub fn cell_mut(&mut self, world_pos: WorldPosition) -> Option<&mut Cell> {
        assert!(self.is_pos_valid(world_pos), "invalid pos {}", &world_pos);
        let (chunk_index, chunk_pos) = self.world_pos_to_chunk_info(world_pos);
        let chunk_o = self.get_chunk_mut(chunk_index);
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
        let bottom_right = top_left + dimensions;

        let chunk_size = self.chunk_size;
        let (chunk_index, mut chunk_pos) = self.world_pos_to_chunk_info(top_left);
        let starter_chunk_x = chunk_pos.x;

        while chunk_pos.y < bottom_right.y {
            while chunk_pos.x < bottom_right.x {
                debug!(self.logger, "pos: {}, right: {}", chunk_pos, bottom_right);
                let chunk_o = self.get_chunk(chunk_pos);
                if let Some(chunk) = chunk_o {
                    for (cell_chunk_pos, cell) in chunk.iter() {
                        let cell_world_pos = chunk.world_position(chunk_index, cell_chunk_pos);
                        if cell_world_pos >= top_left && cell_world_pos <= bottom_right {
                            callback(cell_world_pos, cell);
                        }
                    }
                } 
                chunk_pos.x += chunk_size as i32;
            }
            chunk_pos.y += chunk_size as i32;
            chunk_pos.x = starter_chunk_x;
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_world_pos_to_chunk_pos() {
        let chunk_size = 128;
        let world = World::new(chunk_size as u32);
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
        let world = World::new(chunk_size as u32);
        assert_eq!(
            world.world_pos_to_chunk_index(Point::new(0, 0)),
            ChunkIndex::new(0, 0)
        );
        assert_eq!(
            world.world_pos_to_chunk_index(Point::new(1, 1)),
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
