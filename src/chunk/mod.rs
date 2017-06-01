pub mod generator;
mod index;
mod pos;
pub mod serial;

pub use self::index::*;
pub use self::pos::*;

use graphics::cell::Cell;
use point::Point;

/// Represents a piece of terrain that is part of a larger World. Looking up
/// cells in a World will resolve to a certain Chunk, but actors don't need to
/// care about the underlying Chunks.
#[derive(Serialize, Deserialize)]
pub struct Chunk {
    cells: Vec<Cell>,
}

pub const CHUNK_WIDTH: i32 = 16;

impl Chunk {
    fn index(&self, pos: ChunkPosition) -> usize {
        (pos.0.y * CHUNK_WIDTH + pos.0.x) as usize
    }

    /// Gets an immutable cell reference relative to within this Chunk.
    pub fn cell(&self, pos: ChunkPosition) -> &Cell {
        let index = self.index(pos.into());
        &self.cells[index]
    }

    /// Gets a mutable cell reference relative to within this Chunk.
    pub fn cell_mut(&mut self, pos: ChunkPosition) -> &mut Cell {
        let index = self.index(pos.into());
        &mut self.cells[index]
    }

    /// Calculates the position in the world the point in the chunk represents.
    pub fn world_position_at(index: &ChunkIndex, pos: &ChunkPosition) -> Point {
        Point::new(pos.0.x + index.0.x * CHUNK_WIDTH, pos.0.y + index.0.y * CHUNK_WIDTH)
    }
}
