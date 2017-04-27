use world::{EcsWorld, WorldPosition};
use world::traits::Query;

#[derive(Eq, PartialEq, Copy, Clone)]
pub enum Walkability {
    MonstersWalkable,
    MonstersBlocking,
}

impl Walkability {
    pub fn can_walk(&self, world: &EcsWorld, pos: &WorldPosition) -> bool {
        match *self {
            Walkability::MonstersWalkable => { true },
            Walkability::MonstersBlocking => { world.mob_at(*pos).is_none() }
        }
    }
}
