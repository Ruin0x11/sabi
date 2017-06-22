use std::collections::HashSet;

use calx_ecs::Entity;

use chunk::ChunkIndex;
use data::dungeon;
use point::{Point, RectangleIter};
use prefab::{self, Prefab};
use world::{MapId, World};

pub type GlobalEcs = self::Ecs;

#[derive(Serialize, Deserialize)]
pub struct Globals {
    pub ecs: GlobalEcs,
}

pub trait GlobalEntityQuery {
    fn position(&self, globals: &Globals) -> Option<Point>;
}

impl GlobalEntityQuery for Entity {
    fn position(&self, globals: &Globals) -> Option<Point> {
        globals.ecs.positions.get(*self).cloned()
    }
}

impl Globals {
    pub fn new() -> Self {
        Globals { ecs: GlobalEcs::new() }
    }

    fn find_objects<F>(&self, condition: F) -> Vec<Entity>
    where
        F: FnMut(&&Entity) -> bool,
    {
        self.ecs.iter().filter(condition).cloned().collect()
    }

    fn find_object<F>(&self, condition: F) -> Option<Entity>
    where
        F: FnMut(&&Entity) -> bool,
    {
        self.find_objects(condition).first().cloned()
    }

    fn objects_at<F>(&self, pos: Point, mut condition: F) -> Vec<Entity>
    where
        F: FnMut(&&Entity) -> bool,
    {
        self.find_objects(|e| condition(e) && e.position(self).map_or(false, |p| p == pos))
    }

    fn object_at<F>(&self, pos: Point, condition: F) -> Option<Entity>
    where
        F: FnMut(&&Entity) -> bool,
    {
        self.objects_at(pos, condition).first().cloned()
    }
}

// Second ECS (more like data storage) for global values, to be kept at a higher level than the
// game world. This makes keeping track of data that is persistent between individual maps easier,
// because the ECS for each world is serialized separately.
//
// Note that since no transfering/recreating of entities in the global ECS are made, it is safe to
// embed them as unique identifiers inside other data structures, because the entity ID will never
// change.
Ecs! {
    positions: self::Position,
    dungeons: self::Dungeon,
    towns: self::Town,
}

type Position = Point;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dungeon {
    pub data: dungeon::Dungeon,
    pub placed: bool,
}

impl Dungeon {
    pub fn new() -> Self {
        Dungeon {
            data: dungeon::DungeonPlan::easy().build(),
            placed: false,
        }
    }
}

impl Globals {
    // TODO: please let there be conservative_impl_trait
    pub fn dungeons(&self) -> Vec<Entity> {
        self.find_objects(|&&e| self.ecs.dungeons.get(e).is_some())
    }

    pub fn dungeon_at_mut(&mut self, pos: Point) -> Option<&mut Dungeon> {
        let entity = self.dungeons()
                         .into_iter()
                         .find(|&e| e.position(self).map_or(false, |p| p == pos));

        match entity {
            Some(e) => self.ecs.dungeons.get_mut(e),
            None => None,
        }
    }

    pub fn dungeon_for_map_id(&self, id: MapId) -> Option<Entity> {
        for dungeon in self.dungeons().into_iter() {
            let dungeon_compo = self.ecs.dungeons.get(dungeon).unwrap();
            if dungeon_compo.data.has_floor(id) {
                return Some(dungeon);
            }
        }

        None
    }

    pub fn make_dungeon(&mut self) {
        Loadout::new()
            .c(Position::new(-10, 10))
            .c(Dungeon::new())
            .make(&mut self.ecs);
    }
}



#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Town {
    pub size: Point,
    pub placed: bool,
}

impl Town {
    pub fn new(size: Point) -> Self {
        assert!(size.x > 0);
        assert!(size.y > 0);
        Town {
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

    pub fn generate(&self, offset: Position) -> Prefab {
        let prefab_args = prefab_args! {
            width: self.size.x,
            height: self.size.y,
        };

        prefab::create("town", &Some(prefab_args)).unwrap()
    }
}

impl Globals {
    pub fn towns(&self) -> Vec<Entity> {
        self.find_objects(|&&e| self.ecs.towns.get(e).is_some())
    }

    pub fn town_at_mut(&mut self, pos: Point) -> Option<&mut Town> {
        let entity = self.object_at(pos, |&&e| self.ecs.towns.get(e).is_some());

        match entity {
            Some(e) => self.ecs.towns.get_mut(e),
            None => None,
        }
    }

    pub fn make_town(&mut self) {
        Loadout::new()
            .c(Position::new(0, 0))
            .c(Town::new(Point::new(50, 50)))
            .make(&mut self.ecs);
    }
}
