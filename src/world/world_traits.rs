use world::MapId;
use world::WorldPosition;
use graphics::cell::{CellFeature, StairDest, StairDir};
use chunk::ChunkIndex;
use data::Walkability;
use graphics::cell::Cell;
use world::EcsWorld;
use world::traits::*;

use point::Point;

pub trait WorldQuery {
    fn can_walk(&self, pos: Point, walkability: Walkability) -> bool;
    fn pos_valid(&self, pos: &Point) -> bool;

    fn with_cells<F>(&self, top_left: Point,
                     dimensions: Point,
                     callback: F)
        where F: FnMut(Point, &Cell);
}

pub trait WorldMutate {
    fn cell_mut(&mut self, pos: &Point) -> Option<&mut Cell>;
    fn cell(&mut self, pos: &Point) -> Option<&Cell>;
}

impl WorldMutate for EcsWorld {
    fn cell_mut(&mut self, pos: &Point) -> Option<&mut Cell> {
        if !self.pos_valid(pos) {
            return None;
        }

        let idx = ChunkIndex::from(*pos);
        if !self.terrain().chunk_loaded(&idx) {
            self.load_chunk(&idx).expect("Chunk load failed!");
        }

        self.terrain_mut().cell_mut(pos)
    }

    fn cell(&mut self, pos: &Point) -> Option<&Cell> {
        if !self.pos_valid(pos) {
            return None;
        }

        let idx = ChunkIndex::from(*pos);
        if !self.terrain().chunk_loaded(&idx) {
            self.load_chunk(&idx).expect("Chunk load failed!");
        }

        self.terrain().cell(pos)
    }
}

impl EcsWorld {
    pub fn find_stairs_in(&mut self) -> Option<WorldPosition> {
        let stairs_in = self.terrain.stairs_in.clone();
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
