#[cfg(test)]
mod tests;
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
use rand::{Rng, thread_rng};
use slog::Logger;

use chunk::*;
use chunk::generator::ChunkType;
use chunk::serial::SerialChunk;
use data::spatial::{Spatial, Place};
use data::{TurnOrder, Walkability, MessageLog};
use ecs;
use ecs::*;
use ecs::traits::*;
use graphics::Marks;
use graphics::cell::{CellFeature, StairDir, StairDest, StairKind};
use log;
use logic::entity::*;
use point::{Direction, Point, POINT_ZERO};
use prefab::{self, Prefab, PrefabArgs, PrefabMarker};
use terrain::Terrain;
use terrain::regions::Regions;
use terrain::traits::*;
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

pub struct WorldBuilder {
    bounds: Bounds,
    chunk_type: ChunkType,
    prefab_name: Option<String>,
    prefab_args: Option<PrefabArgs>,
    id: u32,
    max_id: Option<u32>,
    seed: u32,
}

impl World {
    pub fn new() -> WorldBuilder {
        WorldBuilder {
            bounds: Bounds::Unbounded,
            chunk_type: ChunkType::Blank,
            prefab_name: None,
            prefab_args: None,
            id: 0,
            max_id: None,
            seed: 1,
        }
    }

    pub fn deploy_prefab(&mut self, prefab: &Prefab, offset: Point) {
        debug!(self.logger, "About to deploy prefab on map {}...", self.flags().map_id);

        for (pos, cell) in prefab.iter() {
            let offset_pos = pos + offset;
            if let Some(cell_mut) = self.cell_mut(&offset_pos) {
                *cell_mut = *cell;
            }
        }

        for (pos, marker) in prefab.markers.iter() {
            let offset_pos = *pos + offset;
            debug!(self.logger, "Marker: {:?} {}", marker, offset_pos);
            match *marker {
                PrefabMarker::Npc => {
                    self.create(ecs::prefab::npc("dude"), offset_pos);
                },
                // TODO: Allow both stair directions
                PrefabMarker::StairsOut => self.place_stairs_down(*pos, StairKind::Unconnected),
                _ => (),
            }
        }

        self.terrain.markers = prefab.markers.clone();

        debug!(self.logger, "Finished deploying prefab");
    }
}

impl WorldBuilder {
    pub fn build(&mut self) -> Result<World, String> {
        let mut prefab_opt = None;

        if let Some(ref prefab_name) = self.prefab_name {
            let prefab = prefab::create(prefab_name, &self.prefab_args)
                .map_err(|e| e.to_string())?;
            self.bounds = Bounds::Bounded(prefab.width(), prefab.height());
            prefab_opt = Some(prefab);
        }

        let mut world = World {
            ecs_: Ecs::new(),
            terrain: Terrain::new(self.bounds.clone(), self.id),
            spatial: Spatial::new(),
            turn_order: TurnOrder::new(),
            flags: Flags::new(self.seed, self.id),
            chunk_type: self.chunk_type.clone(),

            logger: get_world_log(),
            messages: MessageLog::new(),
            marks: Marks::new(),
            debug_overlay: Marks::new(),
        };

        if let Some(max_id) = self.max_id {
            world.flags_mut().globals.max_map_id = max_id;
        }

        if let Some(prefab) = prefab_opt {
            world.deploy_prefab(&prefab, POINT_ZERO);
        }

        Ok(world)
    }

    pub fn from_other_world<'a>(&'a mut self, other: &World) -> &'a mut Self {
        let next_id = other.flags().get_globals().max_map_id + 1;
        self.id = next_id;
        self.max_id = Some(next_id);
        self.seed = other.flags().seed();
        self
    }

    pub fn with_id<'a>(&'a mut self, id: u32) -> &'a mut Self {
        self.id = id;
        self
    }

    pub fn with_randomized_seed<'a>(&'a mut self) -> &'a mut Self {
        self.seed = thread_rng().next_u32();
        self
    }

    pub fn with_chunk_type<'a>(&'a mut self, chunk_type: ChunkType) -> &'a mut Self {
        self.chunk_type = chunk_type;
        self
    }

    pub fn with_bounds<'a>(&'a mut self, bounds: Bounds) -> &'a mut Self {
        self.bounds = bounds;
        self
    }

    pub fn with_prefab<'a>(&'a mut self, prefab_name: &str) -> &'a mut Self {
        self.prefab_name = Some(prefab_name.to_string());
        self
    }

    pub fn with_prefab_args<'a>(&'a mut self, prefab_args: PrefabArgs) -> &'a mut Self {
        self.prefab_args = Some(prefab_args);
        self
    }
}

#[derive(Serialize, Deserialize)]
pub struct World {
    ecs_: Ecs,
    terrain: Terrain,
    spatial: Spatial,
    turn_order: TurnOrder,
    flags: Flags,

    chunk_type: ChunkType,

    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    #[serde(default = "get_world_log")]
    pub logger: Logger,

    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    #[serde(default = "MessageLog::new")]
    messages: MessageLog,

    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    #[serde(default = "Marks::new")]
    pub marks: Marks,

    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    #[serde(default = "Marks::new")]
    pub debug_overlay: Marks,
}

impl World {
    pub fn get_messages(&self, count: usize) -> Vec<String> {
        self.messages.get_lines(count)
    }

    #[cfg(not(test))]
    pub fn message(&mut self, text: &str) {
        self.messages.append(text);
    }

    #[cfg(test)]
    pub fn message(&mut self, text: &str) {
        println!("MESSAGE: {}", text);
    }

    pub fn next_message(&mut self) {
        self.messages.next_line();
    }
}

impl Query for World {
    fn position(&self, e: Entity) -> Option<WorldPosition> {
        match self.spatial.get(e) {
            Some(Place::At(loc)) => Some(loc),
            // Some(Place::In(container)) => self.position(container),
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

    fn seed(&self) -> u32 {
        self.flags.seed()
    }

    fn entities(&self) -> slice::Iter<Entity> {
        self.ecs_.iter()
    }

    fn entities_at(&self, loc: WorldPosition) -> Vec<Entity> {
        self.spatial.entities_at(loc)
    }

    fn entities_in(&self, entity: Entity) -> Vec<Entity> {
        self.spatial.entities_in(entity)
    }

    fn ecs(&self) -> &Ecs {
        &self.ecs_
    }

    fn flags(&self) -> &Flags {
        &self.flags
    }

    fn turn_order(&self) -> &TurnOrder {
        &self.turn_order
    }
}

impl Mutate for World {
    fn set_entity_location(&mut self, e: Entity, pos: Point) {
        self.spatial.insert_at(e, pos);
    }

    fn place_entity_in(&mut self, container: Entity, e: Entity) {
        self.spatial.insert_in(e, container);
    }

    fn set_player(&mut self, player: Option<Entity>) {
        self.flags.globals.player = player;
    }

    fn kill_entity(&mut self, e: Entity) {
        debug!(self.logger, "Marking entity {:?} as killed.", e);
        self.spatial.remove(e);

        let result = self.turn_order.remove(e);
        if let Err(err) = result {
            warn!(self.logger, "{:?} wasn't in world turn order! {:?}", e, err);
        }
    }

    fn unload_entity(&mut self, e: Entity) -> Loadout {
        debug!(self.logger, "Unloading entity {:?}", e);
        let loadout = Loadout::get(self.ecs(), e);
        self.kill_entity(e);
        self.remove_entity(e);
        loadout
    }

    fn remove_entity(&mut self, e: Entity) {
        debug!(self.logger, "Removing {:?} from world.", e);
        self.ecs_.remove(e);
    }

    fn ecs_mut(&mut self) -> &mut Ecs {
        &mut self.ecs_
    }

    fn flags_mut(&mut self) -> &mut Flags {
        &mut self.flags
    }

    fn move_entity(&mut self, e: Entity, dir: Direction) -> Result<(), ()> {
        let loc = self.position(e).ok_or(())? + dir;
        if self.can_walk(loc, Walkability::MonstersBlocking) {
            self.place_entity(e, loc);
            return Ok(());
        }

        Err(())
    }

    fn next_entity_in_turn_order(&mut self) -> Option<Entity> {
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
            const FOV_RADIUS: i32 = 12;

            let visible = fov::bresenham_fast(self, center, FOV_RADIUS);

            let mut fov = &mut self.ecs_.fovs[e];
            fov.visible = visible;

        }
    }

    fn spawn(&mut self, loadout: &Loadout, pos: WorldPosition) -> Entity {
        let entity = loadout.make(&mut self.ecs_);
        self.place_entity(entity, pos);

        if self.ecs().turns.has(entity) {
            self.turn_order.insert(entity, 0).unwrap();
        }

        debug_ecs!(self, entity, "Spawned entity at {}", pos);

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
            if self.ecs().turns.has(id) {
                self.turn_order.advance_time_for(id, ticks).unwrap();
            }
        }
    }

    fn add_delay_for(&mut self, id: Entity, amount: i32) {
        self.turn_order.add_delay_for(id, amount).unwrap();
    }
}

const UPDATE_RADIUS: i32 = 3;

fn is_persistent(world: &World, entity: Entity) -> bool {
    world.player().map_or(false, |p| entity == p)
}

impl<'a> ChunkedWorld<'a, ChunkIndex, SerialChunk, Regions, Terrain> for World {
    fn terrain(&self) -> &Terrain {
        &self.terrain
    }
    fn terrain_mut(&mut self) -> &mut Terrain {
        &mut self.terrain
    }

    fn load_chunk_internal(&mut self,
                           chunk: SerialChunk,
                           index: &ChunkIndex)
                           -> Result<(), SerialError> {
        // debug!(self.logger, "LOAD CHUNK: {}", index);
        self.terrain.insert_chunk(*index, chunk.chunk);

        let entities = self.frozen_in_chunk(index);
        for e in entities {
            self.spatial.unfreeze(e);
            let result = self.turn_order.resume(e);
            if let Err(err) = result {
                if self.is_mob(e) {
                    warn_ecs!(self, e, "{:?} is mob, but wasn't in turn order! {:?}", e, err);
                }
            }
        }

        Ok(())
    }

    fn unload_chunk_internal(&mut self, index: &ChunkIndex) -> Result<SerialChunk, SerialError> {
        // debug!(
        //     self.logger,
        //     "UNLOAD CHUNK: {} MapId: {}",
        //     index,
        //     self.flags().map_id
        // );
        let chunk = self.terrain
                        .remove_chunk(index)
                        .expect(&format!("Expected chunk at {}!", index));

        let entities = self.entities_in_chunk(index);
        for e in entities {
            if is_persistent(self, e) {
                continue;
            }

            self.spatial.freeze(e);
            let result = self.turn_order.pause(e);
            if let Err(err) = result {
                if self.is_mob(e) {
                    warn_ecs!(self, e, "{:?} is mob, but wasn't in turn order! {:?}", e, err);
                }
            }

        }

        // debug!(
        //     self.logger,
        //     "Chunk ready for serializing, MapId: {}",
        //     self.flags().map_id
        // );

        let serial = SerialChunk { chunk: chunk };
        Ok(serial)
    }

    fn generate_chunk(&mut self, index: &ChunkIndex) -> SerialResult<()> {
        // debug!(
        //     self.logger,
        //     "GEN: {} {:?} reg: {}, map: {}",
        //     index,
        //     self.chunk_type,
        //     self.terrain.id,
        //     self.flags().map_id
        // );
        self.terrain
            .insert_chunk(*index, self.chunk_type.generate(index, self.flags.seed()));

        Ok(())
    }

    fn save(&mut self) -> Result<(), SerialError> {
        let indices = self.terrain.chunk_indices();
        debug!(self.logger, "Saving world...");
        for index in indices.iter() {
            // debug!(self.logger, "SAVE/UNLOAD: {}", index);
            self.unload_chunk(index)?;
        }
        Ok(())
    }
}

impl World {
    pub fn update_chunks(&mut self, center: Point) -> Result<(), SerialError> {
        let mut relevant: HashSet<ChunkIndex> = HashSet::new();

        let start = ChunkIndex::from_world_pos(center);

        relevant.insert(start);
        let quadrant = |dx, dy, idxes: &mut HashSet<ChunkIndex>| for dr in 1..UPDATE_RADIUS + 1 {
            for i in 0..dr + 1 {
                let ax = start.0.x + (dr - i) * dx;
                let ay = start.0.y + i * dy;
                let chunk_idx = ChunkIndex::new(ax, ay);
                idxes.insert(chunk_idx);
            }
        };
        quadrant(-1, 1, &mut relevant);
        quadrant(1, 1, &mut relevant);
        quadrant(-1, -1, &mut relevant);
        quadrant(1, -1, &mut relevant);

        for idx in relevant.iter() {
            if self.terrain().index_in_bounds(idx) && !self.terrain.chunk_loaded(idx) {
                // debug!(self.logger, "LOAD CHUNK: {} MapId {}", idx, self.map_id());
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
            None => POINT_ZERO,
        };

        self.update_chunks(center).unwrap();
    }

    pub fn on_load(&mut self) {
        self.update_terrain();
        self.recalc_entity_fovs();
        self.update_camera();
    }
}

impl World {
    pub fn place_stairs_down(&mut self, pos: WorldPosition, kind: StairKind) {
        assert!(self.can_walk(pos, Walkability::MonstersWalkable));
        self.terrain.cell_mut(&pos).unwrap().feature =
            Some(CellFeature::Stairs(StairDir::Descending, StairDest::Ungenerated(kind)));
    }
}
