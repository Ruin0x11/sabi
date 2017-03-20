use ::Actor;
use world::ChunkIndex;
use glyph::Glyph;
use tile::*;
use point::Point;

type ChunkPosition = Point;

/// Represents a piece of terrain.
/// Monsters/items/instanced things are not kept here. They go in 'World' instead.
pub struct Chunk {
    index: Option<ChunkIndex>,
    dimensions: Point,
    cells: Vec<Cell>,
    actors: Vec<Actor>,
}

impl Chunk {
    // TEMP: You'd normally want to generate a World and then draw rooms, paths,
    // etc. on top of that, because there is no way to see other Chunks at this
    // level.
    pub fn generate_basic(size: u32) -> Chunk {
        let size_i = size as i32;
        let mut cells = Vec::new();

        for index in 0..(size * size) {
            cells.push(Cell {
                tile: Tile {
                    type_: TileType::Floor,
                    glyph: Glyph::Floor,
                    feature: None,
                }})
        }

        Chunk {
            dimensions: Point::new(size_i, size_i),
            cells: cells,
            actors: Vec::new(),
            index: None,
        }
    }

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

    pub fn cell(&self, pos: ChunkPosition) -> &Cell {
        let index = self.index(pos.into());
        &self.cells[index]
    }

    pub fn cell_mut(&mut self, pos: ChunkPosition) -> &mut Cell {
        let index = self.index(pos.into());
        &mut self.cells[index]
    }

    pub fn world_position(&self, index: ChunkIndex, pos: ChunkPosition) -> Point {
        Point::new(pos.x + index.x * self.dimensions.x, pos.y + index.y * self.dimensions.y)
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
