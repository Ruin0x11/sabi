use std::collections::{HashSet, HashMap};
use std::fs::File;

use world::regions::Regions;

use chunk::*;
use chunk::serial::SerialChunk;
use cell;

use ecs::traits::*;

use infinigen::*;

impl Index for ChunkIndex {
    fn x(&self) -> i32 { self.0.x }
    fn y(&self) -> i32 { self.0.y }
}

#[derive(Serialize, Deserialize)]
pub struct Terrain {
    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    #[serde(default="Regions::new")]
    regions: Regions,

    chunks: HashMap<ChunkIndex, Chunk>,
}

impl Terrain {
    pub fn new() -> Self {
        Terrain {
            regions: Regions::new(),
            chunks: HashMap::new(),
        }
    }
}

pub fn get_region_filename(index: &RegionIndex) -> String {
    format!("r.{}.{}.sr", index.0, index.1)
}

impl TerrainQuery for Terrain {
    fn chunk(&self, index: ChunkIndex) -> Option<&Chunk> {
        self.chunks.get(&index)
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
    }

    fn remove_chunk(&mut self, index: &ChunkIndex) -> Option<Chunk> {
        self.chunks.remove(index)
    }
}

impl<'a> ChunkedTerrain<'a, ChunkIndex, SerialChunk, Regions> for Terrain
    where Region<ChunkIndex>: ManagedRegion<'a, ChunkIndex, SerialChunk> {
    fn regions_mut(&mut self) -> &mut Regions {
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
