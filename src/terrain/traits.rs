use graphics::cell::{Cell, CellFeature};
use point::Point;
use infinigen::Index;
use chunk::ChunkIndex;
use world::WorldPosition;

use chunk::*;

// TODO: These should all be private. Let world mutation happen on a single
// uniform API.

pub trait BoundedTerrain<P, I>
    where P: Into<Point>,
          I: Index {
    fn in_bounds(&self, pos: &WorldPosition) -> bool;
    fn index_in_bounds(&self, index: &ChunkIndex) -> bool;
}

/// Queries that are directly related to the terrain itself, and not the
/// entities on top of it.
pub trait TerrainQuery: BoundedTerrain<WorldPosition, ChunkIndex> {
    fn chunk(&self, index: ChunkIndex) -> Option<&Chunk>;
    fn pos_loaded(&self, pos: &WorldPosition) -> bool;

    fn chunk_from_world_pos(&self, pos: WorldPosition) -> Option<&Chunk> {
        let index = ChunkIndex::from_world_pos(pos);
        self.chunk(index)
    }

    fn cell(&self, world_pos: &WorldPosition) -> Option<&Cell> {
        if !self.in_bounds(world_pos) {
            return None;
        }

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

pub trait TerrainMutate: BoundedTerrain<WorldPosition, ChunkIndex> {
    fn prune_empty_regions(&mut self);

    fn insert_chunk(&mut self, index: ChunkIndex, chunk: Chunk);
    fn remove_chunk(&mut self, index: &ChunkIndex) -> Option<Chunk>;
    fn chunk_mut(&mut self, index: ChunkIndex) -> Option<&mut Chunk>;

    fn chunk_mut_from_world_pos(&mut self, pos: WorldPosition) -> Option<&mut Chunk> {
        let index = ChunkIndex::from_world_pos(pos);
        self.chunk_mut(index)
    }

    fn cell_mut(&mut self, world_pos: &WorldPosition) -> Option<&mut Cell> {
        if !self.in_bounds(world_pos) {
            return None;
        }

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
