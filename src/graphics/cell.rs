use std::collections::HashMap;

use toml::Value;

use point::Point;
use util::toml::*;
use world::MapId;

#[derive(Deserialize)]
struct CellData {
    name: String,
    tile: String,
    seethrough: bool,
    passable: bool,
}

struct CellTable {
    indices: HashMap<String, usize>,
    cells: HashMap<usize, CellData>,
}

impl CellTable {
    pub fn get(&self, idx: usize) -> &CellData {
        self.cells.get(&idx).unwrap()
    }

    pub fn get_index(&self, type_: &str) -> usize {
        match self.indices.get(type_) {
            Some(idx) => *idx,
            None => *self.indices.get("error").unwrap(),
        }
    }
}

fn make_cell_data_table() -> CellTable {
    let mut cells = HashMap::new();
    let mut indices = HashMap::new();
    let val = toml_value_from_file("data/cells.toml");

    let cell_table = match val {
        Value::Table(ref table) => table,
        _ => panic!("Cell table wasn't a table."),
    };

    let cell_array = match cell_table["cells"] {
        Value::Array(ref array) => array,
        _ => panic!("Cell array wasn't an array."),
    };

    for (idx, cell) in cell_array.iter().enumerate() {
        let name: String = expect_value_in_table(&cell, "name");
        let tile: String = expect_value_in_table(&cell, "tile");
        let seethrough: bool = expect_value_in_table(&cell, "seethrough");
        let passable: bool = expect_value_in_table(&cell, "passable");

        let data = CellData {
            name: name.clone(),
            tile: tile,
            seethrough: seethrough,
            passable: passable,
        };

        indices.insert(name, idx);
        cells.insert(idx, data);
    }

    CellTable {
        indices: indices,
        cells: cells,
    }
}

lazy_static! {
    static ref CELL_TABLE: CellTable = make_cell_data_table();
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Copy, Clone)]
pub enum StairDir {
    Ascending,
    Descending,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Copy, Clone)]
pub enum StairKind {
    Dungeon(u64),
    DungeonBranch(u64, usize),
    Unconnected,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Copy, Clone)]
pub enum StairDest {
    Ungenerated(StairKind),
    Generated(MapId, Point),
}

use self::StairDir::*;

impl StairDir {
    pub fn reverse(&self) -> StairDir {
        match *self {
            Ascending => Descending,
            Descending => Ascending,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub enum CellFeature {
    Door(bool),
    Stairs(StairDir, StairDest),
}

impl CellFeature {
    pub fn glyph(&self) -> &'static str {
        match *self {
            CellFeature::Door(..) => "door",
            CellFeature::Stairs(StairDir::Ascending, _) => "stairs_up",
            CellFeature::Stairs(StairDir::Descending, _) => "stairs_down",
        }
    }
}

use self::CellFeature::*;

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct Cell {
    pub type_: usize,

    pub feature: Option<CellFeature>,
}

fn get_cell(type_: usize) -> &'static CellData {
    CELL_TABLE.get(type_)
}

impl Cell {
    pub fn new(type_: &str) -> Cell {
        Cell {
            type_: CELL_TABLE.get_index(type_),
            feature: None,
        }
    }

    pub fn set(&mut self, type_: &str) {
        self.type_ = CELL_TABLE.get_index(type_);
    }

    pub fn can_see_through(&self) -> bool {
        get_cell(self.type_).seethrough
    }

    pub fn can_pass_through(&self) -> bool {
        get_cell(self.type_).passable
    }

    pub fn stair_dest_pos(&self) -> Option<Point> {
        match self.feature {
            Some(Stairs(_, StairDest::Generated(_, pos))) => Some(pos),
            _ => None,
        }
    }

    pub fn name(&self) -> &'static str {
        &get_cell(self.type_).name
    }

    pub fn glyph(&self) -> &'static str {
        &get_cell(self.type_).tile
    }
}
