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

impl Terrain {
    // TODO: see infinigen about generalizing this away
    pub fn load_chunk_from_save(&mut self, index: &ChunkIndex) -> Result<(), SerialError> {
        let old_count = self.chunks.len();
        let region = self.regions.get_for_chunk(index);
        let _chunk: SerialChunk = match region.read_chunk(index) {
            Ok(c) => c,
            Err(e) => return Err(e),
        };
        // println!("Loading chunk at {}", index);


        assert_eq!(self.chunks.len(), old_count + 1, "Chunk wasn't inserted into world!");

        Ok(())
    }

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
}

const UPDATE_RADIUS: i32 = 3;

impl<'a> ChunkedTerrain<'a, ChunkIndex, SerialChunk, Regions> for Terrain
    where Region<ChunkIndex>: ManagedRegion<'a, ChunkIndex, SerialChunk> {
    fn load_chunk_internal(&mut self, chunk: SerialChunk, index: &ChunkIndex) -> Result<(), SerialError> {
        self.chunks.insert(index.clone(), Chunk::generate_basic(cell::FLOOR));

        Ok(())
    }

    fn unload_chunk_internal(&mut self, index: &ChunkIndex) -> Result<SerialChunk, SerialError> {
        self.chunks.remove(index);

        let serial = SerialChunk {
            i: 0,
        };
        Ok(serial)
    }

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
