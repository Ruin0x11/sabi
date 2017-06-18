use calx_ecs::Entity;

use data::dungeon;
use point::Point;
use world::MapId;

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
}

pub type GlobalEcs = self::Ecs;

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
        Globals {
            ecs: GlobalEcs::new(),
        }
    }

    fn find_objects<F>(&self, condition: F) -> Vec<Entity>
        where
        F: FnMut(&&Entity) -> bool,
    {
        self.ecs.iter()
            .filter(condition)
            .cloned()
            .collect()
    }

    fn find_object<F>(&self, condition: F) -> Option<Entity>
        where
        F: FnMut(&&Entity) -> bool,
    {
        self.find_objects(condition).first().cloned()
    }
}

impl Globals {
    // TODO: please let there be conservative_impl_trait
    pub fn dungeons(&self) -> Vec<Entity> {
        self.find_objects(|&&e| self.ecs.dungeons.get(e).is_some())
    }

    pub fn dungeon_at_mut(&mut self, pos: Point) -> Option<&mut Dungeon> {
        let entity = self.dungeons().into_iter().find(|&e| e.position(self).map_or(false, |p| p == pos));

        match entity {
            Some(e) => self.ecs.dungeons.get_mut(e),
            None    => None,
        }
    }

    pub fn dungeon_for_map_id(&self, id: MapId) -> Option<Entity> {
        for dungeon in self.dungeons().into_iter() {
            let dungeon_compo = self.ecs.dungeons.get(dungeon).unwrap();
            if dungeon_compo.data.has_floor(id) {
                return Some(dungeon)
            }
        }

        None
    }

    pub fn make_dungeon(&mut self) {
        Loadout::new()
            .c(Position::new(0, 0))
            .c(Dungeon::new())
            .make(&mut self.ecs);
    }
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
