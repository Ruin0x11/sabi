use std::slice;

pub use world::terrain_traits::*;
// TODO: infinigen::traits::*;
pub use infinigen::*;

use calx_ecs::Entity;

use logic::CommandResult;
use graphics::cell::Cell;
use data::{TurnOrder, Walkability};
use point::Direction;
use ecs::*;
use ecs::prefab::*;
use world::WorldPosition;
use world::flags::Flags;

use point::Point;
use chunk::*;

// FIXME: Move these appropriately.
// FIXME: Refactor.

pub trait WorldQuery {
    fn can_walk(&self, pos: Point, walkability: Walkability) -> bool;
    fn pos_valid(&self, pos: &Point) -> bool;

    fn with_cells<F>(&self, top_left: Point,
                     dimensions: Point,
                     callback: F)
        where F: FnMut(Point, &Cell);
}

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

pub trait Mutate: Query + Sized {
    fn set_entity_location(&mut self, e: Entity, loc: Point);

    fn set_player(&mut self, player: Option<Entity>);

    /// Mark an entity as dead, but don't remove it from the system yet.
    fn kill_entity(&mut self, e: Entity);

    /// Remove an entity from the system.
    ///
    /// You generally do not want to call this directly. Mark the entity as dead and it will be
    /// removed at the end of the turn.
    fn remove_entity(&mut self, e: Entity);

    fn place_entity(&mut self, e: Entity, loc: Point) {
        self.set_entity_location(e, loc);
        self.after_entity_moved(e);
    }

    fn move_entity(&mut self, e: Entity, dir: Direction) -> CommandResult;

    fn next_entity(&mut self) -> Option<Entity>;

    fn after_entity_moved(&mut self, e: Entity) {
        self.do_fov(e);
    }

    fn do_fov(&mut self, e: Entity);

    fn ecs_mut<'a>(&'a mut self) -> &'a mut Ecs;

    fn flags_mut<'a>(&'a mut self) -> &'a mut Flags;

    fn spawn(&mut self, loadout: &Loadout, pos: Point) -> Entity;
    fn create(&mut self, prefab: Prefab, pos: Point) -> Entity {
        self.spawn(&prefab.loadout, pos)
    }

    fn kill(&mut self, entity: Entity);

    /// Marks entities as dead based on health. Does not remove the entities
    /// from the system.
    fn update_killed(&mut self) {
        let kill_list: Vec<Entity> =
            self.entities().filter(|&&e| {
                self.ecs().healths.map_or(false, |h| h.is_dead(), e)
            }).cloned().collect();

        for e in kill_list.into_iter() {
            self.kill_entity(e);
        }
    }

    /// Remove destroyed entities from the system.
    fn purge_dead(&mut self) {
        let kill_list: Vec<Entity> =
            self.entities().filter(|&&e| !self.is_alive(e)).cloned().collect();

        for e in kill_list.into_iter() {
            self.remove_entity(e);
        }
    }

    fn advance_time(&mut self, ticks: i32);

    fn add_delay_for(&mut self, id: Entity, amount: i32);

    // Drop in a predefined room.
    // fn create_terrain_prefab()
}

pub trait ComponentQuery<C: Component> {
    /// Gets the component off this entity or panics.
    fn get_or_err(&self, e: Entity) -> &C;

    /// Gets a component off this entity and runs a callback, with a fallback
    /// value if it doesn't exist.
    fn map_or<F, T>(&self, default: T, callback: F, e: Entity) -> T
        where F: FnOnce(&C,) -> T;

    fn map<F, T>(&self, callback: F, e: Entity) -> Option<T>
        where F: FnOnce(&C,) -> T;

    fn has(&self, e: Entity) -> bool;
}

pub trait ComponentMutate<C: Component> {
    fn map_mut<F, T>(&mut self, callback: F, e: Entity) -> Option<T>
        where F: FnOnce(&mut C,) -> T;
}
