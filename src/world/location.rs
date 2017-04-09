use std::collections::HashMap;

use world::*;

const CHUNK_SIZE: i32 = 32;

pub struct Location {
    chunk_size: i32,
    chunks: HashMap<ChunkIndex, Chunk>,
    type_: WorldType,
}


pub enum WorldType {
    Instanced(Point),
    Overworld,
    Nothing
}

impl Location {
    pub fn new(type_: WorldType) -> Self {
        Location {
            chunk_size: CHUNK_SIZE,
            chunks: HashMap::new(),
            type_: type_,
        }
    }

    pub fn generate(type_: WorldType, tile: Tile) -> Self {
        let chunks = match type_ {
            WorldType::Instanced(dimensions) => Location::generate_chunked(CHUNK_SIZE, dimensions, tile),
            _   => HashMap::new(),
        };

        let mut world = Location::new(type_);
        world.chunks = chunks;

        assert!(world.chunks.len() > 0, "No chunks created!");

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
                chunks.insert(ChunkIndex(Point::new(i, j)), Chunk::generate_basic(chunk_size, tile));
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

        ChunkIndex(Point::new(conv(pos.x), conv(pos.y)))
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
