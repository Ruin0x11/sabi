use action::Action;
use calx_ecs::{ComponentData, Entity};
use chunk::*;
use command::CommandResult;
use direction::Direction;
use ecs::*;
use ecs::*;
use ecs::flags::Flags;
use ecs::prefab::*;
use ecs::spatial::*;
use ecs::traits::*;
use ecs::traits::*;
use infinigen::*;
use logic;
use point::Point;
use std::collections::HashSet;
use std::fs::File;
use std::slice;
use world::*;
use world::ChunkIndex;
use world::TurnOrder;
use world::WorldPosition;
use ecs::chunk_management::*;
use tile;

#[derive(Serialize, Deserialize)]
pub struct EcsWorld {
    ecs: Ecs,
    terrain: Terrain,
    spatial: Spatial,
    turn_order: TurnOrder,
    flags: Flags,
}

impl EcsWorld {
    pub fn new(seed: u32) -> EcsWorld {
        EcsWorld {
            ecs: Ecs::new(),
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
        true
    }

    fn seed(&self) -> u32 { self.flags.seed }

    fn entities(&self) -> slice::Iter<Entity> { self.ecs.iter() }

    fn entities_at(&self, loc: WorldPosition) -> Vec<Entity> { self.spatial.entities_at(loc) }

    fn ecs<'a>(&'a self) -> &'a Ecs { &self.ecs }

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

    fn remove_entity(&mut self, e: Entity) { self.ecs.remove(e); }

    fn move_entity(&mut self, e: Entity, dir: Direction) -> CommandResult {
        let loc = try!(self.position(e).ok_or(())) + dir;
        if self.can_enter(e, loc) {
            self.place_entity(e, loc);
            return Ok(());
        }

        Err(())
    }

    fn spawn(&mut self, loadout: &Loadout, pos: WorldPosition) -> Entity {
        let entity = loadout.make(&mut self.ecs);
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

const UPDATE_RADIUS: i32 = 3;

impl<'a> Chunked<'a, File, ChunkIndex, SerialChunk, Region<ChunkIndex>> for EcsWorld {
    fn load_chunk(&mut self, index: &ChunkIndex) -> Result<(), SerialError> {
        if let Err(_) = self.terrain.load_chunk_from_save(index) {
            if self.chunk_loaded(index) {
                return Err(ChunkAlreadyLoaded(index.0.x, index.0.y));
            }
            // println!("Addding chunk at {}", index);
            self.terrain.insert_chunk(index.clone(), Chunk::generate_basic(tile::FLOOR));

            // The region this chunk was created in needs to know of the chunk
            // that was created in-game but nonexistent on disk.
            self.terrain.notify_chunk_creation(index);
        }
        Ok(())
    }

    fn unload_chunk(&mut self, index: &ChunkIndex) -> SerialResult<()> {
        self.terrain.unload_chunk(index)
    }

    fn chunk_loaded(&self, index: &ChunkIndex) -> bool {
        self.terrain.chunk(*index).is_some()
    }

    fn chunk_indices(&self) -> Vec<ChunkIndex> {
        self.terrain.chunk_indices()
    }

    fn update_chunks(&mut self) -> Result<(), SerialError>{
        let mut relevant: HashSet<ChunkIndex> = HashSet::new();
        let center = ChunkIndex::from_world_pos(Point::new(0, 0));
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
            if !self.chunk_loaded(idx) {
                // println!("Loading chunk {}", idx);
                self.load_chunk(idx)?;
            }
        }

        let indices = self.chunk_indices();
        for idx in indices.iter() {
            if !relevant.contains(idx) && self.chunk_loaded(idx) {
                self.unload_chunk(idx)?;
            }
        }

        self.terrain.prune_empty_regions();

        Ok(())
    }

    fn save(mut self) -> Result<(), SerialError> {
        let indices = self.chunk_indices();
        for index in indices.iter() {
            self.unload_chunk(index)?;
        }
        Ok(())
    }
}

impl WorldQuery for EcsWorld {
    fn can_walk(&self, pos: Point, walkability: Walkability) -> bool {
        true
    }

    fn with_cells<F>(&mut self, top_left: Point,
                     dimensions: Point,
                     mut callback: F) where F: FnMut(Point, &Cell) {
        
    }
}

impl<C: Component> ComponentQuery<C> for ComponentData<C> {
    fn get_or_err(&self, e: Entity) -> &C {
        self.get(e).unwrap()
    }
    fn get_or<F, T>(&self, e: Entity, default: T, callback: F) -> T
        where F: FnOnce(&C,) -> T {
        self.get(e).map_or(default, callback)
    }
}
