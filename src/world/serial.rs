use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::PathBuf;

use bincode::{self, Infinite};

use infinigen::*;
use chunk::generator::ChunkType;
use point::Point;
use world::{Bounds, EcsWorld};
use world::traits::Mutate;

pub const SAVE_DIRECTORY: &'static str = "save";

pub fn get_save_directory(id: u32) -> PathBuf {
    PathBuf::from(format!("{}/{}/", SAVE_DIRECTORY, id))
}

fn get_world_savefile(id: u32) -> PathBuf {
    let mut save_dir = get_save_directory(id);
    save_dir.push("world.bin");
    save_dir
}

fn get_manifest_file() -> PathBuf {
    PathBuf::from(format!("{}/manifest.bin", SAVE_DIRECTORY))
}

// TODO: Allow quicksaving, as in not unloading the entire world first
pub fn save_world(world: &mut EcsWorld) -> SerialResult<()> {
    let indices = world.terrain.chunk_indices();
    for index in indices.iter() {
        world.unload_chunk(index)?;
    }

    let data = bincode::serialize(&world, Infinite)?;
    let id = world.flags.map_id;

    let save_path = get_world_savefile(id);

    let mut savefile = File::create(save_path).map_err(SerialError::from)?;
    savefile.write(data.as_slice()).map_err(SerialError::from)?;
    Ok(())
}

pub fn load_world(id: u32) -> SerialResult<EcsWorld> {
    fs::create_dir_all(get_save_directory(id)).map_err(SerialError::from)?;

    let save_path = get_world_savefile(id);

    let mut data: Vec<u8> = Vec::new();
    let mut savefile = File::open(save_path)?;
    savefile.read_to_end(&mut data)?;
    let mut world: EcsWorld = bincode::deserialize(&data)?;

    // TODO: shouldn't have to set manually.
    world.flags_mut().map_id = id;
    world.terrain.set_id(id);

    Ok(world)
}

pub fn save_manifest(world: &EcsWorld) -> SerialResult<()> {
    let manifest = SaveManifest {
        map_id: world.flags.map_id,
        max_map_id: world.flags.max_map_id,
        seed: world.flags.seed,
    };

    let data = bincode::serialize(&manifest, Infinite)?;

    let manifest_path = get_manifest_file();
    let mut manifest = File::create(manifest_path).map_err(SerialError::from)?;
    manifest.write(data.as_slice()).map_err(SerialError::from)?;
    Ok(())
}

pub fn load_manifest() -> SerialResult<SaveManifest> {
    let manifest_path = get_manifest_file();

    let mut data: Vec<u8> = Vec::new();
    let mut savefile = File::create(manifest_path)?;
    savefile.read_to_end(&mut data)?;
    let manifest = bincode::deserialize(&data)?;

    Ok(manifest)
}

pub fn init_paths() -> SerialResult<()> {
    fs::create_dir_all(SAVE_DIRECTORY).map_err(SerialError::from)
}

/// Global save data not tied to any specific map.
#[derive(Serialize, Deserialize)]
pub struct SaveManifest {
    pub map_id: u32,
    pub max_map_id: u32,
    pub seed: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serial_integrity() {

    }
}

