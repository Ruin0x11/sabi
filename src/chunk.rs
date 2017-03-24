use ::Actor;
use world::ChunkIndex;
use glyph::Glyph;
use tile::*;
use point::Point;

type ChunkPosition = Point;

/// Represents a piece of terrain that is part of a larger World. Looking up
/// cells in a World will resolve to a certain Chunk, but actors don't need to
/// care about the underlying Chunks.
pub struct Chunk {
    index: Option<ChunkIndex>,
    dimensions: Point,
    cells: Vec<Cell>,
}

impl Chunk {
    pub fn generate_basic(size: i32, tile: Tile) -> Chunk {
        let mut cells = Vec::new();

        for index in 0..(size * size) {
            cells.push( Cell { tile: tile.clone() } );
        }

        Chunk {
            dimensions: Point::new(size, size),
            cells: cells,
            index: None,
        }
    }

    /// Converts a regular Point into a ChunkPosition.
    /// The Point must be within the size of the Chunk.
    pub fn chunk_point(&self, pos: Point) -> ChunkPosition {
        assert!(pos.x >= 0);
        assert!(pos.y >= 0);
        assert!(pos.x < self.dimensions.x);
        assert!(pos.y < self.dimensions.y);
        ChunkPosition::new(pos.x, pos.y)
    }

    fn index(&self, pos: ChunkPosition) -> usize {
        (pos.y * self.dimensions.x + pos.x) as usize
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
        Point::new(pos.x + index.0.x * self.dimensions.x, pos.y + index.0.y * self.dimensions.y)
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
        let level_position = ChunkPosition::new(x, y);
        self.index += 1;
        match self.inner.next() {
            Some(cell) => {
                Some((level_position, cell))
            }
            None => None,
        }
    }
}
