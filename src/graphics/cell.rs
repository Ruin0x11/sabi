use point::Point;
use world::MapId;

macro_attr! {
    #[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Copy, Clone, EnumFromStr!)]
    pub enum CellType {
        Wall,
        Floor,
        Grass,
        Sand,
        Air,
        Important,
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

    pub fn glyph(&self) -> &'static str {
        self.get_appearance()
    }

    fn get_appearance(&self) -> &'static str {
        match self.type_ {
            CellType::Wall  => "water",
            CellType::Grass  => "grass",
            CellType::Sand  => "sand",
            CellType::Floor => "stone_road",
            _               => "stone_road",
        }
    }
}

// TEMP: A tile ID is all that should be needed, not type and glyph
pub const WALL: Cell = Cell {
    type_: CellType::Wall,
    feature: None,
};

pub const GRASS: Cell = Cell {
    type_: CellType::Grass,
    feature: None,
};

pub const SAND: Cell = Cell {
    type_: CellType::Sand,
    feature: None,
};

pub const DECOR: Cell = Cell {
    type_: CellType::Floor,
    feature: None,
};

pub const FLOOR: Cell = Cell {
    type_: CellType::Floor,
    feature: None,
};

pub const NOTHING: Cell = Cell {
    type_: CellType::Air,
    feature: None,
};
