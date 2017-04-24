use std::collections::HashSet;
use std::fmt;

use ::Actor;
use world::ChunkIndex;
use glyph::Glyph;
use tile::*;
use point::Point;

#[derive(Hash, Eq, PartialEq, Debug, Clone)]
pub struct ChunkPosition(pub Point);

impl From<Point> for ChunkPosition {
    fn from(pos: Point) -> ChunkPosition {
        assert!(pos.x >= 0);
        assert!(pos.y >= 0);
        assert!(pos.x < CHUNK_WIDTH);
        assert!(pos.y < CHUNK_WIDTH);
        ChunkPosition(pos)
    }
}

impl ChunkPosition {
    pub fn from_world(pos: &Point) -> ChunkPosition {
        let conv = |i: i32| {
            let i_new = i % CHUNK_WIDTH;
            if i_new < 0 {
                CHUNK_WIDTH + i_new
            } else {
                i_new
            }
        };
        ChunkPosition(Point::new(conv(pos.x), conv(pos.y)))
    }
}

impl fmt::Display for ChunkPosition {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub type FieldOfView = HashSet<ChunkPosition>;

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
    pub fn generate_basic(tile: Tile) -> Chunk {
        let mut cells = Vec::new();

        for index in 0..(CHUNK_WIDTH * CHUNK_WIDTH) {
            cells.push( Cell { tile: tile.clone() } );
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

    /// Gets an mutable cell reference relative to within this Chunk.
    pub fn cell_mut(&mut self, pos: ChunkPosition) -> &mut Cell {
        let index = self.index(pos.into());
        &mut self.cells[index]
    }

    /// Calculates the position in the world the point in the chunk represents.
    pub fn world_position(&self, index: ChunkIndex, pos: ChunkPosition) -> Point {
        Point::new(pos.0.x + index.0.x * self.dimensions.x, pos.0.y + index.0.y * self.dimensions.y)
    }

    /// Calculates the position in the world the point in the chunk represents.
    pub fn world_position_at(index: &ChunkIndex, pos: &ChunkPosition) -> Point {
        Point::new(pos.0.x + index.0.x * CHUNK_WIDTH, pos.0.y + index.0.y * CHUNK_WIDTH)
    }

    pub fn world_position_from_index(index: ChunkIndex, size: i32) -> Point {
        Point::new(index.0.x * size, index.0.y * size)
    }

    pub fn iter(&self) -> Cells {
        Cells {
            index: 0,
            width: self.dimensions.x,
            inner: self.cells.iter(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Cell {
    pub tile: Tile,
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
