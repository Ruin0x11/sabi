use graphics::Glyph;
use point::Point;
use world::MapId;

macro_attr! {
    #[derive(Serialize, Deserialize, Debug, Copy, Clone, EnumFromStr!)]
    pub enum CellType {
        Wall,
        Floor,
        Air,
        Water,
        Lava
    }
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Copy, Clone)]
pub enum StairDir {
    Ascending,
    Descending,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Copy, Clone)]
pub enum StairDest {
    Ungenerated,
    Generated(MapId, Point),
}

use self::StairDir::*;

impl StairDir {
    pub fn reverse(&self) -> StairDir {
        match *self {
            Ascending  => Descending,
            Descending => Ascending,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub enum CellFeature {
    Door(bool),
    Stairs(StairDir, StairDest),
}

use self::CellFeature::*;

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct Cell {
    pub type_: CellType,

    // TEMP: Shouldn't be owned, but instead looked up
    pub glyph: Glyph,

    pub feature: Option<CellFeature>,
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

    pub fn stair_dest_pos(&self) -> Option<Point> {
        match self.feature {
            Some(Stairs(_, StairDest::Generated(_, pos))) => Some(pos),
            _                                             => None,
        }
    }

    pub fn glyph(&self) -> Glyph {
        match self.feature {
            Some(Stairs(dir, _)) => match dir {
                Ascending  => Glyph::StairsUp,
                Descending => Glyph::StairsDown,
            },
            _ => self.glyph
        }
    }
}

// TEMP: A tile ID is all that should be needed, not type and glyph
pub const WALL: Cell = Cell {
    type_: CellType::Wall,
    glyph: Glyph::Wall,
    feature: None,
};

// TEMP: A tile ID is all that should be needed, not type and glyph
pub const DECOR: Cell = Cell {
    type_: CellType::Floor,
    glyph: Glyph::Fancy,
    feature: None,
};

pub const FLOOR: Cell = Cell {
    type_: CellType::Floor,
    glyph: Glyph::Floor,
    feature: None,
};

pub const NOTHING: Cell = Cell {
    type_: CellType::Air,
    glyph: Glyph::None,
    feature: None,
};
