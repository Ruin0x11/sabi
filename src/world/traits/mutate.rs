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
use world::traits::Query;

use point::Point;
use chunk::*;

pub trait Mutate: Query + Sized {
    fn set_entity_location(&mut self, e: Entity, loc: Point);

    fn set_player(&mut self, player: Option<Entity>);

    /// Mark an entity as dead, but don't remove it from the system yet.
    fn kill_entity(&mut self, e: Entity);

    /// Save an entity's data and remove it from the world cleanly;
    fn unload_entity(&mut self, e: Entity) -> Loadout;

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
