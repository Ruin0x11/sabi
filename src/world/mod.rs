#[cfg(test)] mod tests;
mod bounds;
mod transition;
pub mod flags;
pub mod serial;
pub mod traits;

pub use self::bounds::Bounds;
use self::flags::Flags;
use self::traits::*;

use std::collections::HashSet;
use std::slice;

use calx_ecs::Entity;
use infinigen::*;
use slog::Logger;

use chunk::*;
use chunk::generator::ChunkType;
use chunk::serial::SerialChunk;
use data::spatial::{Spatial, Place};
use data::{TurnOrder, Walkability, MessageLog};
use ecs::*;
use ecs::traits::*;
use graphics::cell::{CellFeature, StairDir, StairDest};
use graphics::{Color, Marks};
use log;
use logic::entity::EntityQuery;
use lua;
use point::{Direction, Point, POINT_ZERO};
use prefab::{Prefab, PrefabMarker};
use terrain::Terrain;
use terrain::traits::*;
use terrain::regions::Regions;
use util::fov;

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

    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    #[serde(default="MessageLog::new")]
    messages: MessageLog,

    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    #[serde(default="Marks::new")]
    pub marks: Marks,

    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    #[serde(default="Marks::new")]
    pub debug_overlay: Marks,
}

impl EcsWorld {
    pub fn new(bounds: Bounds, chunk_type: ChunkType, seed: u32, id: u32) -> EcsWorld {
        EcsWorld {
            ecs_: Ecs::new(),
            terrain: Terrain::new(bounds, id),
            spatial: Spatial::new(),
            turn_order: TurnOrder::new(),
            flags: Flags::new(seed, id),
            chunk_type: chunk_type,

            logger: get_world_log(),
            messages: MessageLog::new(),
            marks: Marks::new(),
            debug_overlay: Marks::new(),
        }
    }

    pub fn from_prefab(prefab: Prefab, seed: u32, id: u32) -> EcsWorld {
        let mut world = EcsWorld::new(Bounds::Bounded(prefab.width(), prefab.height()), ChunkType::Blank, seed, id);

        debug!(world.logger, "About to load prefab over map {}...", world.flags().map_id);

        for (pos, cell) in prefab.iter() {
            if let Some(cell_mut) = world.cell_mut(&pos) {
                *cell_mut = *cell;
            }
            {
                let cellb = world.cell_const(&pos);
                debug!(world.logger, "{}: {:?}, {:?}", pos, cell.type_, cellb);
            }
        }

        for (pos, marker) in prefab.markers.iter() {
            if *marker == PrefabMarker::StairsIn {
                world.terrain.stairs_in.insert(*pos);
            }
        }

        debug!(world.logger, "Finished loading prefab");

        world
    }

    pub fn get_messages(&self, count: usize) -> Vec<String> {
        self.messages.get_lines(count)
    }

    pub fn message(&mut self, text: &str) {
        self.messages.append(text);
    }

    pub fn next_message(&mut self) {
        self.messages.next_line();
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
                    result.push(*e);
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
                    result.push(*e);
                }
            }
        }
        result
    }

    fn seen_entities(&self, viewer: Entity) -> Vec<Entity> {
        if !self.ecs().fovs.has(viewer) {
            return vec![];
        }

        let mut seen = Vec::new();
        for entity in self.entities() {
            if viewer.can_see_other(*entity, self) && *entity != viewer {
                seen.push(*entity);
            }
        }
        seen
    }

    fn seed(&self) -> u32 { self.flags.seed() }

    fn entities(&self) -> slice::Iter<Entity> { self.ecs_.iter() }

    fn entities_at(&self, loc: WorldPosition) -> Vec<Entity> { self.spatial.entities_at(loc) }

    fn ecs(&self) -> &Ecs { &self.ecs_ }

    fn flags(&self) -> &Flags { &self.flags }

    fn turn_order(&self) -> &TurnOrder { &self.turn_order }
}

impl Mutate for EcsWorld {
    fn set_entity_location(&mut self, e: Entity, loc: WorldPosition) { self.spatial.insert_at(e, loc); }

    fn set_player(&mut self, player: Option<Entity>) { self.flags.globals.player = player; }

    fn kill_entity(&mut self, e: Entity) {
        debug_ecs!(self, e, "Removing {:?} from turn order", e);
        self.spatial.remove(e);
        let result = self.turn_order.remove(e);
        if let Err(err) = result {
            warn_ecs!(self, e, "{:?} wasn't in turn order! {:?}", e, err);
        }
    }

    fn unload_entity(&mut self, e: Entity) -> Loadout {
        let loadout = Loadout::get(self.ecs(), e);
        self.kill_entity(e);
        self.remove_entity(e);
        loadout
    }

    fn remove_entity(&mut self, e: Entity) { self.ecs_.remove(e); }

    fn ecs_mut(&mut self) -> &mut Ecs { &mut self.ecs_ }

    fn flags_mut(&mut self) -> &mut Flags { &mut self.flags }

    fn move_entity(&mut self, e: Entity, dir: Direction) -> Result<(), ()> {
        let loc = self.position(e).ok_or(())? + dir;
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
        if !self.ecs().fovs.has(e) {
            return;
        }

        // because FOV is so expensive, monster detection is done
        // through checking for LOS only.
        if !self.is_player(e) {
            return;
        }

        if let Some(center) = self.position(e) {
            const FOV_RADIUS: i32 = 7;

            let visible = fov::bresenham_fast(self, center, FOV_RADIUS);

            let mut fov = &mut self.ecs_.fovs[e];
            fov.visible = visible;

        }
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
const UPDATE_RADIUS: i32 = 3;

fn is_persistent(world: &EcsWorld, entity: Entity) -> bool {
    world.player().map_or(false, |p| entity == p)
}

impl<'a> ChunkedWorld<'a, ChunkIndex, SerialChunk, Regions, Terrain> for EcsWorld {
    fn terrain(&self) -> &Terrain { &self.terrain }
    fn terrain_mut(&mut self) -> &mut Terrain { &mut self.terrain }

    fn load_chunk_internal(&mut self, chunk: SerialChunk, index: &ChunkIndex) -> Result<(), SerialError> {
        debug!(self.logger, "LOAD CHUNK: {}", index);
        self.terrain.insert_chunk(*index, chunk.chunk);

        let entities = self.frozen_in_chunk(index);
        for e in entities {
            self.spatial.unfreeze(e);
            let result = self.turn_order.resume(e);
            if let Err(err) = result {
                warn_ecs!(self, e, "{:?} wasn't in turn order! {:?}", e, err);
            }
        }

        Ok(())
    }

    fn unload_chunk_internal(&mut self, index: &ChunkIndex) -> Result<SerialChunk, SerialError> {
        debug!(self.logger, "UNLOAD CHUNK: {}", index);
        let chunk = self.terrain.remove_chunk(index)
            .expect(&format!("Expected chunk at {}!", index));

        let entities = self.entities_in_chunk(index);
        for e in entities {
            if is_persistent(self, e) {
                continue;
            }

            self.spatial.freeze(e);
            let result = self.turn_order.pause(e);
            if let Err(err) = result {
                warn_ecs!(self, e, "{:?} wasn't in turn order! {:?}", e, err);
            }

        }

        debug!(self.logger, "Id: {}", self.flags().map_id);

        let serial = SerialChunk {
            chunk: chunk,
        };
        Ok(serial)
    }

    fn generate_chunk(&mut self, index: &ChunkIndex) -> SerialResult<()> {
        debug!(self.logger, "GEN: {} {:?} reg: {}, map: {}", index, self.chunk_type, self.terrain.id, self.flags().map_id);
        self.terrain.insert_chunk(*index, self.chunk_type.generate(index, self.flags.seed()));

        let chunk_pos = ChunkPosition::from(Point::new(0, 0));
        let cell_pos = Chunk::world_position_at(index, &chunk_pos);
        if self.can_walk(cell_pos, Walkability::MonstersBlocking) {
            self.create(::ecs::prefab::mob("putit", 10, "putit"),
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

    fn save(&mut self) -> Result<(), SerialError> {
        let indices = self.terrain.chunk_indices();
        debug!(self.logger, "Saving world...");
        for index in indices.iter() {
            debug!(self.logger, "SAVE/UNLOAD: {}", index);
            self.unload_chunk(index)?;
        }
        Ok(())
    }
}

impl EcsWorld {
    pub fn update_chunks(&mut self, center: Point) -> Result<(), SerialError>{
        let mut relevant: HashSet<ChunkIndex> = HashSet::new();

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

    pub fn recalc_entity_fovs(&mut self) {
        if let Some(p) = self.player() {
            self.do_fov(p);
        }
    }

    pub fn update_camera(&mut self) {
        if let Some(player) = self.player() {
            if let Some(pos) = self.position(player) {
                self.flags_mut().camera = pos;
            }
        }
    }

    pub fn update_terrain(&mut self) {
        let center = match self.player() {
            Some(p) => self.position(p).map_or(POINT_ZERO, |p| p),
            None    => POINT_ZERO,
        };

        self.update_chunks(center).unwrap();
    }

    pub fn on_load(&mut self) {
        self.update_terrain();
        self.recalc_entity_fovs();
        self.update_camera();
    }
}
