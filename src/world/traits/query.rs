use std::slice;

// TODO: infinigen::traits::*;
pub use infinigen::*;

use calx_ecs::Entity;

use logic::CommandResult;
use graphics::cell::Cell;
use data::{TurnOrder, Walkability};
use point::Direction;
use ecs::*;
use ecs::prefab::*;
use ecs::traits::*;
use world::WorldPosition;
use world::flags::Flags;

use point::Point;
use chunk::*;

pub trait Query {
    fn position(&self, e: Entity) -> Option<Point>;

    fn is_player(&self, e: Entity) -> bool;

    fn player(&self) -> Option<Entity>;

    fn seed(&self) -> u32;

    fn entities(&self) -> slice::Iter<Entity>;

    fn entities_at(&self, loc: Point) -> Vec<Entity>;

    fn entities_in_chunk(&self, index: &ChunkIndex) -> Vec<Entity>;

    fn frozen_in_chunk(&self, index: &ChunkIndex) -> Vec<Entity>;

    fn ecs<'a>(&'a self) -> &'a Ecs;

    fn flags<'a>(&'a self) -> &'a Flags;

    fn turn_order<'a>(&'a self) -> &'a TurnOrder;

    // FIXME: This is confusing. "Dead" has both the meaning of "not on map" and
    // "health is zero".
    fn is_alive(&self, e: Entity) -> bool;

    fn is_active(&self, e: Entity) -> bool;

    fn can_see(&self, viewer: Entity, pos: WorldPosition) -> bool;

    fn seen_entities(&self, viewer: Entity) -> Vec<Entity>;

    fn is_mob(&self, e: Entity) -> bool {
        let ecs = self.ecs();
        ecs.ais.has(e)
            && ecs.turns.has(e)
            && ecs.healths.has(e)
            && ecs.names.has(e)
            && ecs.fovs.has(e)
    }

    /// Return mob (if any) at given position.
    fn mob_at(&self, loc: Point) -> Option<Entity> {
        self.entities_at(loc).into_iter().find(|&e| self.is_mob(e))
    }

    // fn extract_prefab
}