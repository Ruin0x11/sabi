use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;

use chunk::ChunkIndex;
use chunk::serial::SerialChunk;
use infinigen::*;
use world;

/// Implementation of a region manager.
#[derive(Serialize, Deserialize)]
pub struct Regions {
    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    #[serde(default="HashMap::new")]
    pub regions: HashMap<RegionIndex, Region<ChunkIndex>>,
    pub id: u32,
}

impl Regions {
    pub fn new(id: u32) -> Self {
        Regions {
            regions: HashMap::new(),
            id: id,
        }
    }
    pub fn set_id(&mut self, id: u32) {
        self.id = id;
    }
}

impl Regions {
    fn get_region_filename(index: &RegionIndex) -> String {
        format!("r.{}.{}.sr", index.0, index.1)
    }

    fn get_region_path(&self, index: &RegionIndex) -> PathBuf {
        let mut save_path = world::serial::get_world_save_dir(self.id);
        fs::create_dir_all(&save_path).unwrap();

        save_path.push(&Regions::get_region_filename(index));
        save_path
    }
}

impl<'a> RegionManager<'a, ChunkIndex, SerialChunk> for Regions
    where Region<ChunkIndex>: ManagedRegion<'a, ChunkIndex, SerialChunk>{
    fn load(&mut self, index: RegionIndex) {
        let path = self.get_region_path(&index);

        let handle = Region::get_region_file(path);

        let region = Region {
            handle: Box::new(handle),
            unsaved_chunks: HashSet::new(),
        };

        self.regions.insert(index, region);
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
