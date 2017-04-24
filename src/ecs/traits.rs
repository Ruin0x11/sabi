use std::slice;

use calx_ecs::Entity;

use action::Action;
use direction::Direction;
use ecs::*;
use ecs::flags::Flags;
use ecs::prefab::*;
use command::CommandResult;
use world::*;

use infinigen::SerialResult;
use point::Point;
use chunk::*;

// FIXME: Move these appropriately.

pub trait WorldQuery {
    fn can_walk(&self, pos: Point, walkability: Walkability) -> bool;

    fn with_cells<F>(&mut self, top_left: Point,
                     dimensions: Point,
                     mut callback: F) where F: FnMut(Point, &Cell);
}

/// Queries that are directly related to the terrain itself, and not the
/// entities on top of it.
pub trait TerrainQuery {
    fn chunk(&self, index: ChunkIndex) -> Option<&Chunk>;
    fn chunk_indices(&self) -> Vec<ChunkIndex>;

    fn chunk_from_world_pos(&self, pos: WorldPosition) -> Option<&Chunk> {
        let index = ChunkIndex::from_world_pos(pos);
        self.chunk(index)
    }

    fn cell(&self, world_pos: &WorldPosition) -> Option<&Cell> {
        let chunk_pos = ChunkPosition::from_world(world_pos);
        let chunk_opt = self.chunk_from_world_pos(*world_pos);
        match chunk_opt {
            Some(chunk) => {
                Some(chunk.cell(chunk_pos))
            },
            None => None,
        }
    }
}

pub trait TerrainMutate {
    fn prune_empty_regions(&mut self);

    fn insert_chunk(&mut self, index: ChunkIndex, chunk: Chunk);
    fn unload_chunk(&mut self, index: &ChunkIndex) -> SerialResult<()>;
    fn chunk_mut(&mut self, index: ChunkIndex) -> Option<&mut Chunk>;
    fn notify_chunk_creation(&mut self, index: &ChunkIndex);

    fn chunk_mut_from_world_pos(&mut self, pos: WorldPosition) -> Option<&mut Chunk> {
        let index = ChunkIndex::from_world_pos(pos);
        self.chunk_mut(index)
    }

    fn cell_mut(&mut self, world_pos: &WorldPosition) -> Option<&mut Cell> {
        let chunk_pos = ChunkPosition::from_world(world_pos);
        let chunk_opt = self.chunk_mut_from_world_pos(*world_pos);
        match chunk_opt {
            Some(chunk) => {
                Some(chunk.cell_mut(chunk_pos))
            }
            None => None,
        }
    }
}

pub trait Query {
    fn position(&self, e: Entity) -> Option<Point>;

    fn is_player(&self, e: Entity) -> bool;

    fn player(&self) -> Option<Entity>;

    fn seed(&self) -> u32;

    fn entities(&self) -> slice::Iter<Entity>;

    fn entities_at(&self, loc: Point) -> Vec<Entity>;

    fn ecs<'a>(&'a self) -> &'a Ecs;

    fn flags<'a>(&'a self) -> &'a Flags;

    fn turn_order<'a>(&'a self) -> &'a TurnOrder;

    fn next_entity(&self) -> Option<Entity>;

    fn is_alive(&self, e: Entity) -> bool { self.position(e).is_some() }

    fn is_mob(&self, e: Entity) -> bool { true }

    /// Return mob (if any) at given position.
    fn mob_at(&self, loc: Point) -> Option<Entity> {
        self.entities_at(loc).into_iter().find(|&e| self.is_mob(e))
    }

    /// Return whether the entity can occupy a location.
    fn can_enter(&self, e: Entity, loc: Point) -> bool {
        true
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

    fn after_entity_moved(&mut self, e: Entity) {
        //fov
    }

    fn spawn(&mut self, loadout: &Loadout, pos: Point) -> Entity;
    fn create(&mut self, prefab: Prefab, pos: Point) -> Entity {
        self.spawn(&prefab.loadout, pos)
    }

    fn run_action(&mut self, entity: Entity, action: Action);

    /// Remove destroyed entities from system
    fn purge_dead(&mut self) {
        let kill_list: Vec<Entity> =
            self.entities().filter(|&&e| !self.is_alive(e)).cloned().collect();

        for e in kill_list.into_iter() {
            self.remove_entity(e);
        }
    }

    fn advance_time(&mut self, ticks: i32);

    fn add_delay_for(&mut self, id: &Entity, amount: i32);

    // Drop in a predefined room.
    // fn create_terrain_prefab()
}

pub trait ComponentQuery<C: Component> {
    /// Gets the component off this entity or panics.
    fn get_or_err(&self, e: Entity) -> &C;

    /// Gets a component off this entity and runs a callback, with a fallback
    /// value if it doesn't exist.
    fn get_or<F, T>(&self, e: Entity, default: T, callback: F) -> T
        where F: FnOnce(&C,) -> T;
}
