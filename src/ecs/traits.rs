use std::slice;

use calx_ecs::Entity;

use action::Action;
use direction::Direction;
use ecs::*;
use ecs::flags::Flags;
use ecs::prefab::*;
use command::CommandResult;
use data::{TurnOrder, Walkability};
use cell::{Cell, CellFeature};
use world::WorldPosition;

use infinigen::SerialResult;
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

/// Queries that are directly related to the terrain itself, and not the
/// entities on top of it.
pub trait TerrainQuery {
    fn chunk(&self, index: ChunkIndex) -> Option<&Chunk>;

    fn pos_valid(&self, pos: &WorldPosition) -> bool { self.cell(pos).is_some() }

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
    fn remove_chunk(&mut self, index: &ChunkIndex) -> Option<Chunk>;
    fn chunk_mut(&mut self, index: ChunkIndex) -> Option<&mut Chunk>;

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

    fn set_cell(&mut self, pos: WorldPosition, cell: Cell) {
        // self.debug_cell(&pos);
        if let Some(cell_mut) = self.cell_mut(&pos) {
            *cell_mut = cell;
        }
    }

    fn set_cell_feature(&mut self, pos: &WorldPosition, feature: Option<CellFeature>) {
        if let Some(cell_mut) = self.cell_mut(pos) {
            cell_mut.feature = feature;
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

    fn is_alive(&self, e: Entity) -> bool { self.position(e).is_some() }

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

    fn run_action(&mut self, entity: Entity, action: Action);

    /// Marks entities as dead based on health. Does not remove the entities
    /// from the system.
    fn update_killed(&mut self) {
        let kill_list: Vec<Entity> =
            self.entities().filter(|&&e| !self.is_alive(e)).cloned().collect();

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

    fn add_delay_for(&mut self, id: &Entity, amount: i32);

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
