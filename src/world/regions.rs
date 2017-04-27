use std::collections::{HashMap, HashSet};
use std::fs::File;

use chunk::ChunkIndex;
use chunk::serial::SerialChunk;
use infinigen::*;

/// Implementation of a region manager.
pub struct Regions {
    pub regions: HashMap<RegionIndex, Region<ChunkIndex>>,
}

impl Regions {
    pub fn new() -> Self {
        Regions {
            regions: HashMap::new(),
        }
    }
}

fn get_filename(index: &RegionIndex) -> String {
    format!("r.{}.{}.sr", index.0, index.1)
}

impl<'a> RegionManager<'a, ChunkIndex, SerialChunk> for Regions
    where Region<ChunkIndex>: ManagedRegion<'a, ChunkIndex, SerialChunk>{
    fn load(&mut self, index: RegionIndex) {
        let filename = get_filename(&index);

        let handle = Region::get_region_file(filename);

        let region = Region {
            handle: Box::new(handle),
            unsaved_chunks: HashSet::new(),
        };

        self.regions.insert(index.clone(), region);
    }

    fn region_indices(&self) -> Vec<RegionIndex> {
        self.regions.iter().map(|(i, _)| i).cloned().collect()
    }

    fn get(&mut self, index: &RegionIndex) -> Option<&Region<ChunkIndex>> {
        self.regions.get(index)
    }

    fn get_mut(&mut self, index: &RegionIndex) -> Option<&mut Region<ChunkIndex>> {
        self.regions.get_mut(index)
    }

    fn remove(&mut self, index: &RegionIndex) {
        self.regions.remove(index);
    }

    fn region_loaded(&self, index: &RegionIndex) -> bool {
        self.regions.contains_key(index)
    }
}
