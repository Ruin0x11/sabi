use calx_ecs::Entity;
use infinigen::*;

use chunk::ChunkIndex;
use data::Walkability;
use ecs::Loadout;
use graphics::cell::Cell;
use graphics::cell::{CellFeature, StairDest, StairDir};
use prefab::PrefabMarker;
use terrain::traits::*;
use world::World;
use world::MapId;
use world::WorldPosition;
use world::traits::*;

use point::{Point, RectangleIter};

pub trait WorldQuery {
    fn can_walk(&self, pos: Point, walkability: Walkability) -> bool;

    /// Returns true if the given position has been loaded from disk and is
    /// contained in the terrain structure.
    fn pos_loaded(&self, pos: &Point) -> bool;

    /// Returns true if it is possible to see past the tile for the purpose of
    /// FOV/line of sight.
    fn light_passes_through(&self, pos: &Point) -> bool;

    fn with_cells<F>(&self, top_left: Point,
                     dimensions: Point,
                     callback: F)
        where F: FnMut(Point, &Cell);

    fn cell_const(&self, pos: &Point) -> Option<&Cell>;
}


impl WorldQuery for World {
    fn can_walk(&self, pos: Point, walkability: Walkability) -> bool {
        let cell_walkable = self.terrain.cell(&pos).map_or(false, |c| c.can_pass_through());
        // TODO: Should be anything blocking, like blocking terrain features
        let no_mob = walkability.can_walk(self, &pos);
        cell_walkable && no_mob
    }

    fn pos_loaded(&self, pos: &Point) -> bool {
        self.terrain.pos_loaded(pos)
    }

    fn light_passes_through(&self, pos: &Point) -> bool {
        self.cell_const(&pos).map_or(false, |c| c.can_see_through())
    }

    fn with_cells<F>(&self, top_left: Point,
                     dimensions: Point,
                     mut callback: F) where F: FnMut(Point, &Cell) {
        for point in RectangleIter::new(top_left, dimensions) {
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

impl World {
    fn autoload_chunk(&mut self, pos: &Point) {
        let idx = ChunkIndex::from(*pos);
        // debug!(self.logger, "Chunk loaded at {}: {}", idx, self.terrain().chunk_loaded(&idx));
        if !self.terrain().chunk_loaded(&idx) {
            assert_eq!(self.flags().map_id, self.terrain().id);
            self.load_chunk(&idx).expect("Chunk load failed!");
            self.terrain_mut().regions_mut().notify_chunk_creation(&idx);
        }
    }
}

impl WorldMutate for World {
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

impl World {
    pub fn find_marker(&mut self, kind: PrefabMarker) -> Option<WorldPosition> {
        let markers = self.terrain.markers.clone();

        for (pos, marker) in markers.iter() {
            if *marker == kind {
                if self.cell(pos).is_some() {
                    return Some(*pos)
                }
            }
        }

        None
    }

    pub fn find_stairs_in(&mut self) -> Option<WorldPosition> {
        self.find_marker(PrefabMarker::StairsIn)
    }

    pub fn add_marker_overlays(&mut self) {
        for (pos, marker) in self.terrain.markers.iter() {
            self.debug_overlay.add(*pos, marker.to_mark());
        }
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

    pub fn create(&mut self, loadout: Loadout, pos: Point) -> Option<Entity> {
        if self.pos_loaded(&pos) {
            Some(self.spawn(&loadout, pos))
        } else {
            None
        }
    }
}
