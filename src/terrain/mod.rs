use std::collections::HashMap;

use chunk::*;
use chunk::serial::SerialChunk;
use prefab::Markers;
use world::{Bounds, WorldPosition};

use infinigen::*;

pub mod traits;
pub mod regions;

use self::regions::Regions;
use self::traits::*;

impl BoundedTerrain<WorldPosition, ChunkIndex> for Terrain {
    fn in_bounds(&self, pos: &WorldPosition) -> bool {
        self.bounds.in_bounds(pos)
    }

    fn index_in_bounds(&self, index: &ChunkIndex) -> bool {
        self.bounds.index_in_bounds(index)
    }
}

impl Index for ChunkIndex {
    fn x(&self) -> i32 { self.0.x }
    fn y(&self) -> i32 { self.0.y }
}

#[derive(Serialize, Deserialize)]
pub struct Terrain {
    regions: Regions,

    chunks: HashMap<ChunkIndex, Chunk>,
    bounds: Bounds,

    pub markers: Markers,
    pub id: u32,
}

impl Terrain {
    pub fn new(bounds: Bounds, id: u32) -> Self {
        Terrain {
            regions: Regions::new(id),
            chunks: HashMap::new(),
            bounds: bounds,
            markers: Markers::new(),
            id: id,
        }
    }

    pub fn set_id(&mut self, id: u32) {
        self.id = id;
        self.regions.set_id(id);
    }
}

impl TerrainQuery for Terrain {
    fn chunk(&self, index: ChunkIndex) -> Option<&Chunk> {
        self.chunks.get(&index)
    }

    fn pos_loaded(&self, pos: &WorldPosition) -> bool {
        self.cell(pos).is_some() && self.bounds.in_bounds(pos)
    }

    fn bounds(&self) -> &Bounds {
        &self.bounds
    }
}

impl TerrainMutate for Terrain {
    fn prune_empty_regions(&mut self) {
        self.regions.prune_empty();
    }

    fn chunk_mut(&mut self, index: ChunkIndex) -> Option<&mut Chunk> {
        self.chunks.get_mut(&index)
    }

    fn insert_chunk(&mut self, index: ChunkIndex, chunk: Chunk) {
        self.chunks.insert(index, chunk);
        // NOTE: cells are not cropped at bounds, but since there is a bounds
        // check the squares out of bounds are treated as "None"
    }

    fn remove_chunk(&mut self, index: &ChunkIndex) -> Option<Chunk> {
        self.chunks.remove(index)
    }
}

impl<'a> ChunkedTerrain<'a, ChunkIndex, SerialChunk, Regions> for Terrain
    where Region<ChunkIndex>: ManagedRegion<'a, ChunkIndex, SerialChunk> {
    fn regions_mut(&mut self) -> &mut Regions {
        assert_eq!(self.regions.id, self.id);
        &mut self.regions
    }

    fn chunk_count(&self) -> usize {
        self.chunks.len()
    }

    fn chunk_loaded(&self, index: &ChunkIndex) -> bool {
        self.chunk(*index).is_some()
    }

    fn chunk_indices(&self) -> Vec<ChunkIndex> {
        self.chunks.iter().map(|(&i, _)| i).collect()
    }
}
