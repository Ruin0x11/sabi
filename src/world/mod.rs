mod regions;
mod terrain;

pub use self::terrain::Terrain;
use self::regions::Regions;

use std::collections::HashSet;
use std::fs::File;
use std::slice;

use calx_ecs::{ComponentData, Entity};

use action::Action;
use cell::{self, Cell};
use chunk::*;
use chunk::serial::SerialChunk;
use command::CommandResult;
use data::spatial::{Spatial, Place};
use data::{TurnOrder, Walkability};
use direction::Direction;
use ecs::*;
use ecs::flags::Flags;
use ecs::traits::*;
use infinigen::*;
use logic;
use point::Point;
use point::RectangleArea;

pub type WorldPosition = Point;

pub type WorldIter = Iterator<Item=WorldPosition>;

#[derive(Serialize, Deserialize)]
pub struct EcsWorld {
    ecs_: Ecs,
    terrain: Terrain,
    spatial: Spatial,
    turn_order: TurnOrder,
    flags: Flags,
}

impl EcsWorld {
    pub fn new(seed: u32) -> EcsWorld {
        EcsWorld {
            ecs_: Ecs::new(),
            terrain: Terrain::new(),
            spatial: Spatial::new(),
            turn_order: TurnOrder::new(),
            flags: Flags::new(seed),
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
        self.ecs().healths.map_or(false, |h| h.hit_points > 0, e)
    }

    fn seed(&self) -> u32 { self.flags.seed }

    fn entities(&self) -> slice::Iter<Entity> { self.ecs_.iter() }

    fn entities_at(&self, loc: WorldPosition) -> Vec<Entity> { self.spatial.entities_at(loc) }

    fn ecs<'a>(&'a self) -> &'a Ecs { &self.ecs_ }

    fn flags<'a>(&'a self) -> &'a Flags { &self.flags }

    fn turn_order<'a>(&'a self) -> &'a TurnOrder { &self.turn_order }

    fn next_entity(&self) -> Option<Entity> {
        None
    }
}

impl Mutate for EcsWorld {
    fn set_entity_location(&mut self, e: Entity, loc: WorldPosition) { self.spatial.insert_at(e, loc); }

    fn set_player(&mut self, player: Option<Entity>) { self.flags.player = player; }

    fn kill_entity(&mut self, e: Entity) {
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

    fn do_fov(&mut self, e: Entity) {
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
        self.turn_order.add(entity, 0);
        entity
    }

    fn run_action(&mut self, entity: Entity, action: Action) {
        logic::run_action(self, entity, action);
    }

    fn advance_time(&mut self, ticks: i32) {
        let ids: Vec<Entity> = self.entities()
            .filter(|&&e| self.ecs().turns.get(e).is_some())
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

    fn with_cells<F>(&mut self, top_left: Point,
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

const UPDATE_RADIUS: i32 = 3;

impl<'a> ChunkedWorld<'a, ChunkIndex, SerialChunk, Regions, Terrain> for EcsWorld {
    fn terrain(&mut self) -> &mut Terrain { &mut self.terrain }

    fn load_chunk_internal(&mut self, chunk: SerialChunk, index: &ChunkIndex) -> Result<(), SerialError> {
        self.terrain.insert_chunk(index.clone(), Chunk::generate_basic(cell::FLOOR));

        Ok(())
    }

    fn unload_chunk_internal(&mut self, index: &ChunkIndex) -> Result<SerialChunk, SerialError> {
        let chunk = self.terrain.remove_chunk(index);

        let serial = SerialChunk {
            i: 0,
        };
        Ok(serial)
    }

    fn generate_chunk(&mut self, index: &ChunkIndex) -> SerialResult<()> {
        self.terrain.insert_chunk(index.clone(), Chunk::generate_basic(cell::FLOOR));

        Ok(())
    }

    fn update_chunks(&mut self) -> Result<(), SerialError>{
        let mut relevant: HashSet<ChunkIndex> = HashSet::new();

        let center = ChunkIndex::from_world_pos(self.flags.camera);

        relevant.insert(center);
        let quadrant = |dx, dy, idxes: &mut HashSet<ChunkIndex>| {
            for dr in 1..UPDATE_RADIUS+1 {
                for i in 0..dr+1 {
                    let ax = center.0.x + (dr - i) * dx;
                    let ay = center.0.y + i * dy;
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
                // println!("Loading chunk {}", idx);
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
