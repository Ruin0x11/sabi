pub mod flags;
mod regions;
mod terrain;
pub mod traits;

pub use self::terrain::{Bounds, Terrain};
use self::regions::Regions;
use self::traits::*;
use self::flags::Flags;

use std::collections::HashSet;
use std::slice;

use calx_ecs::{ComponentData, Entity};
use slog::Logger;

use action::Action;
use cell::{self, Cell};
use chunk::*;
use chunk::serial::SerialChunk;
use command::CommandResult;
use data::spatial::{Spatial, Place};
use data::{TurnOrder, Walkability};
use direction::Direction;
use ecs::*;
use infinigen::*;
use log;
use logic;
use point::{POINT_ZERO, Point, RectangleArea};

pub type WorldPosition = Point;

impl WorldPosition {
    pub fn from_chunk_index(index: ChunkIndex) -> Point {
        Point::new(index.0.x * CHUNK_WIDTH, index.0.y * CHUNK_WIDTH)
    }
}

lazy_static! {
    static ref WORLD_LOG: Logger = log::make_logger("world");
}

fn get_world_log() -> Logger {
    WORLD_LOG.new(o!())
}


#[derive(Serialize, Deserialize)]
pub struct EcsWorld {
    ecs_: Ecs,
    terrain: Terrain,
    spatial: Spatial,
    turn_order: TurnOrder,
    flags: Flags,

    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    #[serde(default="get_world_log")]
    logger: Logger,
}

impl EcsWorld {
    pub fn new(bounds: Bounds, seed: u32) -> EcsWorld {
        EcsWorld {
            ecs_: Ecs::new(),
            terrain: Terrain::new(bounds),
            spatial: Spatial::new(),
            turn_order: TurnOrder::new(),
            flags: Flags::new(seed),
            logger: get_world_log(),
        }
    }

    pub fn new_blank(w: i32, h: i32) -> EcsWorld {
        let bounds = Point::new(w, h);
        let mut world = EcsWorld::new(Bounds::Bounded(bounds), 0);
        // FIXME: If chunks aren't loaded, doesn't do anything.
        // Temporarily load the chunks at the given points and unload after.
        for point in RectangleArea::new(POINT_ZERO, bounds) {
            world.terrain.set_cell(point, cell::FLOOR);
        }
        world
    }
}

impl Query for EcsWorld {
    fn position(&self, e: Entity) -> Option<WorldPosition> {
        match self.spatial.get(e) {
            Some(Place::At(loc)) => Some(loc),
            Some(Place::In(container)) => self.position(container),
            _ => None,
        }
    }

    fn player(&self) -> Option<Entity> {
        if let Some(p) = self.flags.player {
            if self.is_alive(p) {
                return Some(p);
            }
        }

        None
    }

    fn is_player(&self, e: Entity) -> bool {
        self.player().map_or(false, |p| p == e)
    }

    fn is_alive(&self, e: Entity) -> bool {
        self.is_active(e)
            && self.ecs().healths.map_or(false, |h| h.hit_points > 0, e)
    }

    fn is_active(&self, e: Entity) -> bool {
        self.position(e).is_some()
    }

    fn entities_in_chunk(&self, index: &ChunkIndex) -> Vec<Entity> {
        let mut result = Vec::new();
        for e in self.entities() {
            if let Some(pos) = self.position(*e) {
                if ChunkIndex::from_world_pos(pos) == *index {
                    result.push(e.clone());
                }
            }
        }
        debug!(self.logger, "In {:?}", result);
        result
    }

    fn frozen_in_chunk(&self, index: &ChunkIndex) -> Vec<Entity> {
        let mut result = Vec::new();
        for (e, p) in self.spatial.iter() {
            if let Place::Unloaded(pos) = *p {
                if ChunkIndex::from_world_pos(pos) == *index {
                    result.push(e.clone());
                }
            }
        }
        debug!(self.logger,"Froz {:?}", result);
        result
    }

    fn can_see(&self, viewer: Entity, pos: WorldPosition) -> bool {
        true
    }

    fn seen_entities(&self, viewer: Entity) -> Vec<Entity> {
        if !self.ecs().fovs.has(viewer) {
            return vec![];
        }

        let mut seen = Vec::new();
        for entity in self.entities() {
            if let Some(pos) = self.position(*entity) {
                if self.can_see(viewer, pos) && *entity != viewer {
                    seen.push(entity.clone());
                }
            }
        }
        seen
    }

    fn seed(&self) -> u32 { self.flags.seed }

    fn entities(&self) -> slice::Iter<Entity> { self.ecs_.iter() }

    fn entities_at(&self, loc: WorldPosition) -> Vec<Entity> { self.spatial.entities_at(loc) }

    fn ecs<'a>(&'a self) -> &'a Ecs { &self.ecs_ }

    fn flags<'a>(&'a self) -> &'a Flags { &self.flags }

    fn turn_order<'a>(&'a self) -> &'a TurnOrder { &self.turn_order }
}

impl Mutate for EcsWorld {
    fn set_entity_location(&mut self, e: Entity, loc: WorldPosition) { self.spatial.insert_at(e, loc); }

    fn set_player(&mut self, player: Option<Entity>) { self.flags.player = player; }

    fn kill_entity(&mut self, e: Entity) {
        debug_ecs!(self, e, "Removing {:?} from turn order", e);
        self.spatial.remove(e);
        self.turn_order.remove(&e)
    }

    fn ecs_mut<'a>(&'a mut self) -> &'a mut Ecs { &mut self.ecs_ }

    fn flags_mut<'a>(&'a mut self) -> &'a mut Flags { &mut self.flags }

    fn remove_entity(&mut self, e: Entity) { self.ecs_.remove(e); }

    fn move_entity(&mut self, e: Entity, dir: Direction) -> CommandResult {
        let loc = try!(self.position(e).ok_or(())) + dir;
        if self.can_walk(loc, Walkability::MonstersBlocking) {
            self.place_entity(e, loc);
            return Ok(());
        }

        Err(())
    }

    fn next_entity(&mut self) -> Option<Entity> {
        self.turn_order.next()
    }

    fn do_fov(&mut self, e: Entity) {
        return;
        if !self.ecs().fovs.has(e) {
            return;
        }

        if let Some(ref center) = self.position(e) {
            const FOV_RADIUS: i32 = 12;

            let ref mut fov = self.ecs_.fovs[e];

            fov.update(&self.terrain, center, FOV_RADIUS);
        }
    }

    fn spawn(&mut self, loadout: &Loadout, pos: WorldPosition) -> Entity {
        let entity = loadout.make(&mut self.ecs_);
        self.place_entity(entity, pos);
        self.turn_order.insert(entity, 0);
        entity
    }

    fn run_action(&mut self, entity: Entity, action: Action) {
        logic::run_action(self, entity, action);
    }

    fn advance_time(&mut self, ticks: i32) {
        let ids: Vec<Entity> = self.entities()
        // TODO: Kludge to avoid removing entities first?
            .filter(|&&e| self.is_active(e) && self.ecs().turns.get(e).is_some())
            .cloned().collect();
        for id in ids {
            self.turn_order.advance_time_for(&id, ticks);
        }
    }

    fn add_delay_for(&mut self, id: &Entity, amount: i32) {
        self.turn_order.add_delay_for(id, amount);
    }
}

impl WorldQuery for EcsWorld {
    fn can_walk(&self, pos: Point, walkability: Walkability) -> bool {
        let cell_walkable = self.terrain.cell(&pos).map_or(false, |c| c.can_pass_through());
        // TODO: Should be anything blocking
        let no_mob = walkability.can_walk(self, &pos);
        cell_walkable && no_mob
    }

    fn pos_valid(&self, pos: &Point) -> bool {
        self.terrain.pos_valid(pos)
    }

    fn with_cells<F>(&self, top_left: Point,
                     dimensions: Point,
                     mut callback: F) where F: FnMut(Point, &Cell) {
        let bottom_right = top_left + dimensions;
        for point in RectangleArea::new(top_left, bottom_right) {
            if let Some(cell) = self.terrain.cell(&point) {
                callback(point, cell);
            }
        }
    }
}

impl<C: Component> ComponentQuery<C> for ComponentData<C> {
    fn get_or_err(&self, e: Entity) -> &C {
        self.get(e).unwrap()
    }
    fn map_or<F, T>(&self, default: T, callback: F, e: Entity) -> T
        where F: FnOnce(&C,) -> T {
        self.get(e).map_or(default, callback)
    }

    fn map<F, T>(&self, callback: F, e: Entity) -> Option<T>
        where F: FnOnce(&C,) -> T {
        self.get(e).map(callback)
    }

    fn has(&self, e: Entity) -> bool {
        self.get(e).is_some()
    }
}

impl<C: Component> ComponentMutate<C> for ComponentData<C> {
    fn map_mut<F, T>(&mut self, callback: F, e: Entity) -> Option<T>
        where F: FnOnce(&mut C,) -> T {
        self.get_mut(e).map(callback)
    }
}

const UPDATE_RADIUS: i32 = 3;

fn is_persistent(world: &EcsWorld, entity: Entity) -> bool {
    world.player().map_or(false, |p| entity == p)
}

impl<'a> ChunkedWorld<'a, ChunkIndex, SerialChunk, Regions, Terrain> for EcsWorld {
    fn terrain(&mut self) -> &mut Terrain { &mut self.terrain }

    fn load_chunk_internal(&mut self, chunk: SerialChunk, index: &ChunkIndex) -> Result<(), SerialError> {
        self.terrain.insert_chunk(index.clone(), chunk.chunk);

        let entities = self.frozen_in_chunk(index);
        for e in entities {
            self.spatial.unfreeze(e);
            self.turn_order.resume(e);
        }

        Ok(())
    }

    fn unload_chunk_internal(&mut self, index: &ChunkIndex) -> Result<SerialChunk, SerialError> {
        let chunk = self.terrain.remove_chunk(index)
            .expect(&format!("Expected chunk at {}!", index));

        let entities = self.entities_in_chunk(index);
        for e in entities {
            if is_persistent(self, e) {
                continue;
            }

            println!("Freezing! {:?}", e);
            self.spatial.freeze(e);
            self.turn_order.pause(e);
        }

        let serial = SerialChunk {
            chunk: chunk,
        };
        Ok(serial)
    }

    fn generate_chunk(&mut self, index: &ChunkIndex) -> SerialResult<()> {
        self.terrain.insert_chunk(index.clone(), Chunk::gen_perlin(index, self.flags.seed));

        Ok(())
    }

    fn update_chunks(&mut self) -> Result<(), SerialError>{
        let mut relevant: HashSet<ChunkIndex> = HashSet::new();

        let center = match self.player() {
            Some(p) => self.position(p).map_or(POINT_ZERO, |p| p),
            None    => POINT_ZERO,
        };

        let start = ChunkIndex::from_world_pos(center);

        relevant.insert(start);
        let quadrant = |dx, dy, idxes: &mut HashSet<ChunkIndex>| {
            for dr in 1..UPDATE_RADIUS+1 {
                for i in 0..dr+1 {
                    let ax = start.0.x + (dr - i) * dx;
                    let ay = start.0.y + i * dy;
                    let chunk_idx = ChunkIndex::new(ax, ay);
                    idxes.insert(chunk_idx);
                }
            }
        };
        quadrant(-1,  1, &mut relevant);
        quadrant(1,   1, &mut relevant);
        quadrant(-1, -1, &mut relevant);
        quadrant(1,  -1, &mut relevant);

        for idx in relevant.iter() {
            if !self.terrain.chunk_loaded(idx) {
                self.load_chunk(idx)?;
            }
        }

        let indices = self.terrain.chunk_indices();
        for idx in indices.iter() {
            if !relevant.contains(idx) && self.terrain.chunk_loaded(idx) {
                self.unload_chunk(idx)?;
            }
        }

        self.terrain.prune_empty_regions();

        Ok(())
    }

    fn save(mut self) -> Result<(), SerialError> {
        let indices = self.terrain.chunk_indices();
        for index in indices.iter() {
            self.unload_chunk(index)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test::*;
    use state;

    #[test]
    fn test_persistence() {
        let mut context = test_context_bounded(64, 64);

        let e = {
            let world_mut = &mut context.state.world;

            place_mob(world_mut, WorldPosition::new(1, 1))
        };

        let world = &context.state.world;

        assert_eq!(is_persistent(world, world.player().unwrap()), true);
        assert_eq!(is_persistent(world, e), false);
    }

    #[test]
    fn test_frozen() {
        let mut context = test_context_bounded(1024, 1024);
        let e = {
            let mut world = &mut context.state.world;

            place_mob(&mut world, WorldPosition::new(1, 1))
        };

        assert!(context.state.world.entities_in_chunk(&ChunkIndex::new(0, 0)).contains(&e));

        state::run_action(&mut context, Action::TeleportUnchecked(WorldPosition::new(1023, 1023)));

        {
            let world = &context.state.world;

            for e in world.entities() {
                println!("{:?}", world.spatial.get(*e));
            }
        }

        assert_eq!(context.state.world.frozen_in_chunk(&ChunkIndex::new(0, 0)), vec![e]);

        state::run_action(&mut context, Action::TeleportUnchecked(WorldPosition::new(0, 0)));
    }
}
