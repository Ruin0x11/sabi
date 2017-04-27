use glyph::{Glyph};

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub enum CellType {
    Wall,
    Floor,
    Air,
    Water,
    Lava
}

impl Cell {
    pub fn can_see_through(&self) -> bool {
        match self.type_ {
            CellType::Wall |
            CellType::Air  => false,
            _              => true,
        }
    }

    pub fn can_pass_through(&self) -> bool {
        match self.type_ {
            CellType::Wall => false,
            _              => true,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub enum CellFeature {
    Door(bool),
    StairsUp,
    StairsDown,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct Cell {
    pub type_: CellType,

    // TEMP: Shouldn't go here, but is instead looked up
    pub glyph: Glyph,

    pub feature: Option<CellFeature>,
}

// TEMP: A tile ID is all that should be needed, not type and glyph
pub const WALL: Cell = Cell {
    type_: CellType::Wall,
    glyph: Glyph::Wall,
    feature: None,
};

pub const FLOOR: Cell = Cell {
    type_: CellType::Floor,
    glyph: Glyph::Floor,
    feature: None,
};

pub const AIR: Cell = Cell {
    type_: CellType::Air,
    glyph: Glyph::None,
    feature: None,
};
