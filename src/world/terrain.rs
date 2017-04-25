use std::collections::{HashSet, HashMap};
use std::fs::File;

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
    #[serde(default="RegionManager::new")]
    regions: RegionManager<ChunkIndex>,

    chunks: HashMap<ChunkIndex, Chunk>,
}

impl Terrain {
    pub fn new() -> Self {
        Terrain {
            regions: RegionManager::new(),
            chunks: HashMap::new(),
        }
    }
}

pub fn get_region_filename(index: &RegionIndex) -> String {
    format!("r.{}.{}.sr", index.0, index.1)
}

impl<'a> Manager<'a, SerialChunk, File, ChunkIndex, Region<ChunkIndex>> for RegionManager<ChunkIndex>
    where Region<ChunkIndex>: ManagedRegion<'a, SerialChunk, File, ChunkIndex>{
    fn load(&self, index: RegionIndex) -> Region<ChunkIndex> {
        assert!(!self.regions.contains_key(&index), "Region already loaded! {}", index);
        let filename = get_region_filename(&index);

        let handle = Region::get_region_file(filename);

        Region {
            handle: Box::new(handle),
            unsaved_chunks: HashSet::new(),
        }
    }

    fn prune_empty(&mut self) {
        let indices: Vec<RegionIndex> = self.regions.iter().map(|(i, _)| i).cloned().collect();
        for idx in indices {
            if self.regions.get(&idx).map_or(false, |r: &Region<ChunkIndex>| r.is_empty()) {
                // println!("UNLOAD REGION {}", idx);
                self.regions.remove(&idx);
            }
        }
    }

    fn get_for_chunk(&mut self, chunk_index: &ChunkIndex) -> &mut Region<ChunkIndex> {
        let region_index = Region::get_region_index(chunk_index);

        if !self.regions.contains_key(&region_index) {
            let region = self.load(region_index);
            self.regions.insert(region_index.clone(), region);
        }

        self.regions.get_mut(&region_index).unwrap()
    }
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

        self.chunks.insert(index.clone(), Chunk::generate_basic(cell::FLOOR));

        assert_eq!(self.chunks.len(), old_count + 1, "Chunk wasn't inserted into world!");

        Ok(())
    }

}

impl TerrainQuery for Terrain {
    fn chunk(&self, index: ChunkIndex) -> Option<&Chunk> {
        self.chunks.get(&index)
    }

    fn chunk_indices(&self) -> Vec<ChunkIndex> {
        self.chunks.iter().map(|(&i, _)| i).collect()
    }
}

impl TerrainMutate for Terrain {
    fn chunk_mut(&mut self, index: ChunkIndex) -> Option<&mut Chunk> {
        self.chunks.get_mut(&index)
    }

    fn insert_chunk(&mut self, index: ChunkIndex, chunk: Chunk) {
        self.chunks.insert(index, chunk);
    }

    fn unload_chunk(&mut self, index: &ChunkIndex) -> SerialResult<()> {
        let chunk = match self.chunks.remove(&index) {
            Some(c) => c,
            None => return Err(NoChunkInWorld(index.0.x, index.0.y)),
        };
        let region = self.regions.get_for_chunk(index);
        region.write_chunk(chunk, index)
    }

    fn notify_chunk_creation(&mut self, index: &ChunkIndex) {
        self.regions.notify_chunk_creation(index);
    }

    fn prune_empty_regions(&mut self) {
        self.regions.prune_empty()
    }
}
