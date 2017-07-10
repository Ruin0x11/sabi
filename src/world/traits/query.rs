use std::slice;

// TODO: infinigen::traits::*;
pub use infinigen::*;

use calx_ecs::Entity;
use uuid;

use chunk::ChunkIndex;
use data::TurnOrder;
use ecs::*;
use ecs::traits::*;
use world::flags::Flags;

use point::Point;

pub trait Query {
    fn position(&self, e: Entity) -> Option<Point>;

    fn is_player(&self, e: Entity) -> bool;

    fn player(&self) -> Option<Entity>;

    fn party(&self) -> Vec<Entity>;

    fn seed(&self) -> u32;

    fn entities(&self) -> slice::Iter<Entity>;

    fn entities_at(&self, loc: Point) -> Vec<Entity>;

    fn entities_in(&self, entity: Entity) -> Vec<Entity>;

    fn entities_in_chunk(&self, index: &ChunkIndex) -> Vec<Entity>;

    fn frozen_in_chunk(&self, index: &ChunkIndex) -> Vec<Entity>;

    fn ecs(&self) -> &Ecs;

    fn flags(&self) -> &Flags;

    fn turn_order(&self) -> &TurnOrder;

    // FIXME: This is confusing. "Dead" has both the meaning of "not on map" and
    // "health is zero".
    fn is_alive(&self, e: Entity) -> bool;

    fn is_active(&self, e: Entity) -> bool;

    fn seen_entities(&self, viewer: Entity) -> Vec<Entity>;

    fn find_entities<F>(&self, loc: Point, condition: F) -> Vec<Entity>
    where
        F: FnMut(&Entity) -> bool,
    {
        self.entities_at(loc)
            .into_iter()
            .filter(condition)
            .collect()
    }

    fn find_entity<F>(&self, loc: Point, condition: F) -> Option<Entity>
    where
        F: FnMut(&Entity) -> bool,
    {
        self.find_entities(loc, condition).first().cloned()
    }

    fn entity_by_uuid(&self, uuid: uuid::Uuid) -> Option<Entity> {
        self.ecs()
            .uuids
            .ent_iter()
            .find(|&&e| self.ecs().uuids.get(e).map_or(false, |u| u.uuid == uuid))
            .cloned()
    }

    fn is_mob(&self, e: Entity) -> bool {
        let ecs = self.ecs();
        ecs.ais.has(e) && ecs.turns.has(e) && ecs.healths.has(e) && ecs.names.has(e)
    }

    fn is_npc(&self, e: Entity) -> bool {
        let ecs = self.ecs();
        self.is_mob(e) && ecs.npcs.has(e)
    }

    fn is_item(&self, e: Entity) -> bool {
        let ecs = self.ecs();
        ecs.items.has(e) && ecs.names.has(e)
    }

    fn tangible_entities_at(&self, loc: Point) -> Vec<Entity> {
        self.find_entities(loc, |&e| {
            self.position(e).is_some() && self.ecs().appearances.get(e).is_some()
        })
    }

    /// Return mob (if any) at given position.
    fn mob_at(&self, loc: Point) -> Option<Entity> {
        self.find_entity(loc, |&e| self.is_mob(e))
    }

    fn entities_below(&self, entity: Entity) -> Vec<Entity> {
        if !self.is_mob(entity) {
            return Vec::new();
        }

        if let Some(pos) = self.position(entity) {
            let mut at_pos = self.entities_at(pos);
            at_pos.retain(|e| *e != entity);
            return at_pos;
        }

        Vec::new()
    }
}
