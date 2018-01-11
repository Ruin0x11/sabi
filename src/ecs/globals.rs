use std::collections::{HashMap, HashSet};

use chunk::ChunkIndex;
use data::dungeon;
use logic::quest::Quest;
use point::{Point, RectangleIter};
use prefab::{self, Prefab};

#[derive(Serialize, Deserialize)]
pub struct Globals {
    pub dungeons: HashMap<u64, Dungeon>,
    dungeon_count: u64,
    pub towns: Vec<Town>,
    pub quests: Vec<Quest>,
}

impl Globals {
    pub fn new() -> Self {
        Globals {
            dungeons: HashMap::new(),
            dungeon_count: 0,
            towns: Vec::new(),
            quests: Vec::new(),
        }
    }
}

type Position = Point;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dungeon {
    pub pos: Position,
    pub data: dungeon::Dungeon,
    pub placed: bool,
}

impl Dungeon {
    pub fn new(pos: Position) -> Self {
        Dungeon {
            pos: pos,
            data: dungeon::DungeonPlan::easy().build(),
            placed: false,
        }
    }
}

impl Globals {
    pub fn make_dungeon(&mut self) {
        self.dungeon_count += 1;
        self.dungeons
            .insert(self.dungeon_count, Dungeon::new(Position::new(-10, 10)));
    }
}



#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Town {
    pub pos: Point,
    pub size: Point,
    pub placed: bool,
}

impl Town {
    pub fn new(pos: Point, size: Point) -> Self {
        assert!(size.x > 0);
        assert!(size.y > 0);
        Town {
            pos: pos,
            size: size,
            placed: false,
        }
    }

    pub fn spanning_chunks(&self, offset: Position) -> HashSet<ChunkIndex> {
        let mut set = HashSet::new();
        for pos in RectangleIter::new(offset, self.size) {
            set.insert(ChunkIndex::from(pos));
        }
        set
    }

    pub fn generate(&self, _offset: Position) -> Prefab {
        let prefab_args =
            prefab_args! {
            width: self.size.x,
            height: self.size.y,
        };

        prefab::create("town", &Some(prefab_args)).unwrap()
    }
}

impl Globals {
    pub fn make_town(&mut self) {
        self.towns
            .push(Town::new(Position::new(0, 0), Point::new(50, 50)));
    }
}
