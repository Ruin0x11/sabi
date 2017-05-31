use infinigen::*;

use chunk::ChunkIndex;
use data::Walkability;
use ecs::traits::*;
use graphics::cell::Cell;
use graphics::cell::{CellFeature, StairDest, StairDir};
use terrain::traits::*;
use world::EcsWorld;
use world::MapId;
use world::WorldPosition;
use world::traits::*;

use point::{Point, RectangleIter};

pub trait WorldQuery {
    fn can_walk(&self, pos: Point, walkability: Walkability) -> bool;

    /// Returns true if the given position has been loaded from disk and is
    /// contained in the terrain structure.
    fn pos_loaded(&self, pos: &Point) -> bool;

    fn with_cells<F>(&self, top_left: Point,
                     dimensions: Point,
                     callback: F)
        where F: FnMut(Point, &Cell);

    fn cell_const(&self, pos: &Point) -> Option<&Cell>;
}


impl WorldQuery for EcsWorld {
    fn can_walk(&self, pos: Point, walkability: Walkability) -> bool {
        let cell_walkable = self.terrain.cell(&pos).map_or(false, |c| c.can_pass_through());
        // TODO: Should be anything blocking, like blocking terrain features
        let no_mob = walkability.can_walk(self, &pos);
        cell_walkable && no_mob
    }

    fn pos_loaded(&self, pos: &Point) -> bool {
        self.terrain.pos_loaded(pos)
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

    fn cell_const(&self, pos: &Point) -> Option<&Cell> {
        if !self.pos_loaded(pos) {
            return None;
        }

        self.terrain().cell(pos)
    }
}

pub trait WorldMutate {
    fn cell_mut(&mut self, pos: &Point) -> Option<&mut Cell>;
    fn cell(&mut self, pos: &Point) -> Option<&Cell>;
}

impl EcsWorld {
    fn autoload_chunk(&mut self, pos: &Point) {
        let idx = ChunkIndex::from(*pos);
        debug!(self.logger, "Chunk loaded at {}: {}", idx, self.terrain().chunk_loaded(&idx));
        if !self.terrain().chunk_loaded(&idx) {
            self.load_chunk(&idx).expect("Chunk load failed!");
            self.terrain_mut().regions_mut().notify_chunk_creation(&idx);
        }
    }
}

impl WorldMutate for EcsWorld {
    fn cell_mut(&mut self, pos: &Point) -> Option<&mut Cell> {
        if !self.terrain().in_bounds(pos) {
            debug!(self.logger, "invalid: {}", pos);
            return None;
        }

        self.autoload_chunk(pos);

        self.terrain_mut().cell_mut(pos)
    }

    fn cell(&mut self, pos: &Point) -> Option<&Cell> {
        if !self.terrain().in_bounds(pos) {
            debug!(self.logger, "invalid: {}", pos);
            return None;
        }

        self.autoload_chunk(pos);

        self.terrain().cell(pos)
    }
}

impl EcsWorld {
    pub fn find_stairs_in(&mut self) -> Option<WorldPosition> {
        let stairs_in = self.terrain.stairs_in.clone();
        debug!(self.logger, "{:?}", stairs_in);
        for pos in stairs_in.iter() {
            if let Some(_) = self.cell(&pos) {
                return Some(*pos)
            }
        }
        None
    }

    pub fn place_stairs(&mut self, dir: StairDir,
                        pos: WorldPosition,
                        leading_to: MapId,
                        dest_pos: WorldPosition) {
        if let Some(cell_mut) = self.cell_mut(&pos) {
            let dest = StairDest::Generated(leading_to, dest_pos);
            cell_mut.feature = Some(CellFeature::Stairs(dir, dest));
        }
    }
}
