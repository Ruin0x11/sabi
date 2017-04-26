mod index;
mod pos;
pub mod serial;

pub use self::index::*;
pub use self::pos::*;

use std::collections::HashSet;

use cell::Cell;
use point::Point;

/// Represents a piece of terrain that is part of a larger World. Looking up
/// cells in a World will resolve to a certain Chunk, but actors don't need to
/// care about the underlying Chunks.
#[derive(Serialize, Deserialize)]
pub struct Chunk {
    dimensions: Point,
    cells: Vec<Cell>,
}

pub const CHUNK_WIDTH: i32 = 16;

impl Chunk {
    pub fn generate_basic(cell: Cell) -> Chunk {
        let mut cells = Vec::new();

        for _ in 0..(CHUNK_WIDTH * CHUNK_WIDTH) {
            cells.push(cell.clone());
        }

        Chunk {
            dimensions: Point::new(CHUNK_WIDTH, CHUNK_WIDTH),
            cells: cells,
        }
    }

    fn index(&self, pos: ChunkPosition) -> usize {
        (pos.0.y * self.dimensions.x + pos.0.x) as usize
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

    pub fn iter(&self) -> Cells {
        Cells {
            index: 0,
            width: self.dimensions.x,
            inner: self.cells.iter(),
        }
    }
}

pub struct Cells<'a> {
    index: i32,
    width: i32,
    inner: ::std::slice::Iter<'a, Cell>,
}

impl<'a> Iterator for Cells<'a> {
    type Item = (ChunkPosition, &'a Cell);

    fn next(&mut self) -> Option<(ChunkPosition, &'a Cell)> {
        let x = self.index % self.width;
        let y = self.index / self.width;
        let level_position = ChunkPosition::from(Point::new(x, y));
        self.index += 1;
        match self.inner.next() {
            Some(cell) => {
                Some((level_position, cell))
            }
            None => None,
        }
    }
}
