pub mod flags;
mod regions;
mod terrain;
mod terrain_traits;
pub mod traits;
pub mod serial;
mod transition;
mod bounds;

pub use self::terrain::Terrain;
pub use self::bounds::Bounds;
use self::regions::Regions;
use self::traits::*;
use self::flags::Flags;

use std::collections::HashSet;
use std::slice;

use calx_ecs::{ComponentData, Entity};
use slog::Logger;

use graphics::Glyph;
use graphics::cell::{CellFeature, Cell, StairDir, StairDest};
use chunk::*;
use chunk::generator::ChunkType;
use chunk::serial::SerialChunk;
use data::spatial::{Spatial, Place};
use data::{TurnOrder, Walkability};
use ecs::*;
use log;
use logic::{Action, CommandResult};
use point::{Direction, POINT_ZERO, Point, RectangleIter};

pub type MapId = u32;
pub type WorldPosition = Point;

impl From<ChunkIndex> for WorldPosition {
    fn from(index: ChunkIndex) -> Point {
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

    chunk_type: ChunkType,

    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    #[serde(default="get_world_log")]
    pub logger: Logger,
}

impl EcsWorld {
    pub fn new(bounds: Bounds, chunk_type: ChunkType, seed: u32) -> EcsWorld {
        EcsWorld {
            ecs_: Ecs::new(),
            terrain: Terrain::new(bounds),
            spatial: Spatial::new(),
            turn_order: TurnOrder::new(),
            flags: Flags::new(seed),
            chunk_type: chunk_type,
            logger: get_world_log(),
        }
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
        if let Some(p) = self.flags.globals.player {
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
        self.ecs().healths.map_or(false, |h| h.hit_points > 0, e)
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
        result
    }

    fn can_see(&self, _viewer: Entity, _pos: WorldPosition) -> bool {
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

    fn seed(&self) -> u32 { self.flags.seed() }

    fn entities(&self) -> slice::Iter<Entity> { self.ecs_.iter() }

    fn entities_at(&self, loc: WorldPosition) -> Vec<Entity> { self.spatial.entities_at(loc) }

    fn ecs<'a>(&'a self) -> &'a Ecs { &self.ecs_ }

    fn flags<'a>(&'a self) -> &'a Flags { &self.flags }

    fn turn_order<'a>(&'a self) -> &'a TurnOrder { &self.turn_order }
}

impl Mutate for EcsWorld {
    fn set_entity_location(&mut self, e: Entity, loc: WorldPosition) { self.spatial.insert_at(e, loc); }

    fn set_player(&mut self, player: Option<Entity>) { self.flags.globals.player = player; }

    fn kill_entity(&mut self, e: Entity) {
        debug_ecs!(self, e, "Removing {:?} from turn order", e);
        self.spatial.remove(e);
        self.turn_order.remove(e);
    }

    fn unload_entity(&mut self, e: Entity) -> Loadout {
        let loadout = Loadout::get(self.ecs(), e);
        self.kill_entity(e);
        self.remove_entity(e);
        loadout
    }

    fn remove_entity(&mut self, e: Entity) { self.ecs_.remove(e); }

    fn ecs_mut<'a>(&'a mut self) -> &'a mut Ecs { &mut self.ecs_ }

    fn flags_mut<'a>(&'a mut self) -> &'a mut Flags { &mut self.flags }

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

    fn do_fov(&mut self, _e: Entity) {
        // if !self.ecs().fovs.has(e) {
        //     return;
        // }

        // if let Some(ref center) = self.position(e) {
        //     const FOV_RADIUS: i32 = 12;

        //     let ref mut fov = self.ecs_.fovs[e];

        //     fov.update(&self.terrain, center, FOV_RADIUS);
        // }
    }

    fn spawn(&mut self, loadout: &Loadout, pos: WorldPosition) -> Entity {
        let entity = loadout.make(&mut self.ecs_);
        self.place_entity(entity, pos);
        self.turn_order.insert(entity, 0).unwrap();
        entity
    }

    fn kill(&mut self, entity: Entity) {
        self.ecs_mut().healths.map_mut(|h| h.kill(), entity);
    }

    fn advance_time(&mut self, ticks: i32) {
        let ids: Vec<Entity> = self.entities()
        // TODO: Kludge to avoid removing entities first?
            .filter(|&&e| self.is_active(e) && self.ecs().turns.get(e).is_some())
            .cloned().collect();
        for id in ids {
            self.turn_order.advance_time_for(id, ticks).unwrap();
        }
    }

    fn add_delay_for(&mut self, id: Entity, amount: i32) {
        self.turn_order.add_delay_for(id, amount).unwrap();
    }
}

impl WorldQuery for EcsWorld {
    fn can_walk(&self, pos: Point, walkability: Walkability) -> bool {
        let cell_walkable = self.terrain.cell(&pos).map_or(false, |c| c.can_pass_through());
        // TODO: Should be anything blocking, like blocking terrain features
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
        for point in RectangleIter::new(top_left, bottom_right) {
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
    fn terrain(&self) -> &Terrain { &self.terrain }
    fn terrain_mut(&mut self) -> &mut Terrain { &mut self.terrain }

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

            self.spatial.freeze(e);
            self.turn_order.pause(e);
        }

        let serial = SerialChunk {
            chunk: chunk,
        };
        Ok(serial)
    }

    fn generate_chunk(&mut self, index: &ChunkIndex) -> SerialResult<()> {
        debug!(self.logger, "GEN: {} {:?} reg: {}", index, self.chunk_type, self.terrain.id);
        self.terrain.insert_chunk(index.clone(), self.chunk_type.generate(index, self.flags.seed()));

        let chunk_pos = ChunkPosition::from(Point::new(0, 0));
        let cell_pos = Chunk::world_position_at(&index, &chunk_pos);
        if self.can_walk(cell_pos, Walkability::MonstersBlocking) {
            self.create(::ecs::prefab::mob("Putit", 10, Glyph::Putit),
                        cell_pos);
        }

        let stair_pos = cell_pos + (0, 1);
        if self.can_walk(stair_pos, Walkability::MonstersWalkable) {
            self.terrain.cell_mut(&stair_pos).unwrap().feature =
                Some(CellFeature::Stairs(StairDir::Descending,
                                         StairDest::Ungenerated));
        }

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
            if self.terrain().index_in_bounds(idx) && !self.terrain.chunk_loaded(idx) {
                debug!(self.logger, "LOAD CHUNK: {} MapId {}", idx, self.map_id());
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
    use testing::*;
    use state;

    #[test]
    fn test_persistence() {
        let mut context = test_context_bounded(64, 64);

        let mob = {
            let world_mut = &mut context.state.world;
            place_mob(world_mut, WorldPosition::new(1, 1))
        };

        let world = &context.state.world;

        assert_eq!(is_persistent(world, world.player().unwrap()), true);
        assert_eq!(is_persistent(world, mob), false);
    }


    #[test]
    fn test_alive_active() {
        let mut context = test_context_bounded(64, 64);
        let mob_pos = WorldPosition::new(1, 1);
        let mob_chunk = ChunkIndex::from_world_pos(mob_pos);

        let mob = {
            let world_mut = &mut context.state.world;
            place_mob(world_mut, mob_pos)
        };

        {
            let world = &context.state.world;
            assert_eq!(world.is_alive(mob), true);
            assert_eq!(world.is_active(mob), true);
            assert_eq!(world.ecs().contains(mob), true);
        }

        context.state.world.unload_chunk(&mob_chunk).unwrap();

        {
            let world = &context.state.world;
            assert_eq!(world.is_alive(mob), true);
            assert_eq!(world.is_active(mob), false);
            assert_eq!(world.ecs().contains(mob), true);
        }

        context.state.world.load_chunk(&mob_chunk).unwrap();
        context.state.world.kill(mob);

        {
            let world = &context.state.world;
            assert_eq!(world.is_alive(mob), false);
            assert_eq!(world.is_active(mob), true);
            assert_eq!(world.ecs().contains(mob), true);
        }

        context.state.world.update_killed();

        {
            let world = &context.state.world;
            assert_eq!(world.is_alive(mob), false);
            assert_eq!(world.is_active(mob), false);
            assert_eq!(world.ecs().contains(mob), true);
        }

        context.state.world.purge_dead();

        {
            let world = &context.state.world;
            assert_eq!(world.is_alive(mob), false);
            assert_eq!(world.is_active(mob), false);
            assert_eq!(world.ecs().contains(mob), false);
        }
    }

    #[test]
    fn test_frozen() {
        let mut context = test_context_bounded(1024, 1024);
        let mob_pos = WorldPosition::new(1, 1);
        let mob_chunk = ChunkIndex::from_world_pos(mob_pos);
        let mob = {
            let mut world = &mut context.state.world;
            place_mob(&mut world, mob_pos)
        };

        assert!(context.state.world.entities_in_chunk(&mob_chunk).contains(&mob));

        state::run_action_no_ai(&mut context, Action::TeleportUnchecked(WorldPosition::new(1023, 1023)));

        assert_eq!(
            context.state.world.frozen_in_chunk(&ChunkIndex::new(0, 0)),
            vec![mob]
        );
        assert_eq!(
            context.state.world.spatial.get(mob),
            Some(Place::Unloaded(mob_pos))
        );

        state::run_action_no_ai(&mut context, Action::TeleportUnchecked(WorldPosition::new(0, 0)));

        assert_eq!(
            context.state.world.position(mob),
            Some(mob_pos)
        );
        assert_eq!(
            context.state.world.spatial.get(mob),
            Some(Place::At(mob_pos))
        );
        assert!(context.state.world.entities_in_chunk(&mob_chunk).contains(&mob));
    }
}
