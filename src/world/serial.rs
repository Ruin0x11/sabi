use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::PathBuf;

use bincode::{self, Infinite};

use infinigen::*;
use world::{EcsWorld, MapId};
use world::traits::Transition;
use world::flags::GlobalFlags;

pub const SAVE_DIRECTORY: &'static str = "save";

fn get_save_directory() -> String {
    if cfg!(test) {
        format!("test/{}", SAVE_DIRECTORY)
    } else {
        SAVE_DIRECTORY.to_string()
    }
}

pub fn get_world_save_dir(id: u32) -> String {
    let savedir = get_save_directory();
    format!("{}/{}/", savedir, id)
}

fn get_world_savefile(id: u32) -> PathBuf {
    let mut save_dir = get_world_save_dir(id);
    save_dir.push_str("world.bin");
    PathBuf::from(save_dir)
}

fn get_manifest_file() -> PathBuf {
    PathBuf::from(format!("{}/manifest.bin", get_save_directory()))
}

// TODO: Allow quicksaving, as in not unloading the entire world first
pub fn save_world(world: &mut EcsWorld) -> SerialResult<()> {
    // Unloads and saves the terrain.
    world.save()?;

    // The following saves entities and world data besides terrain.
    let data = bincode::serialize(&world, Infinite)?;
    let id = world.map_id();

    fs::create_dir_all(get_world_save_dir(id)).map_err(SerialError::from)?;
    let save_path = get_world_savefile(id);

    let mut savefile = File::create(save_path).map_err(SerialError::from)?;
    savefile.write(data.as_slice()).map_err(SerialError::from)?;
    Ok(())
}

pub fn load_world(id: u32) -> SerialResult<EcsWorld> {
    fs::create_dir_all(get_world_save_dir(id)).map_err(SerialError::from)?;

    let save_path = get_world_savefile(id);

    let mut data: Vec<u8> = Vec::new();
    let mut savefile = File::open(save_path)?;
    savefile.read_to_end(&mut data)?;
    let mut world: EcsWorld = bincode::deserialize(&data)?;

    // TODO: shouldn't have to set manually.
    world.set_map_id(id);

    world.on_load();

    Ok(world)
}

pub fn save_manifest(world: &EcsWorld) -> SerialResult<()> {
    let manifest = SaveManifest {
        globals: world.flags.get_globals(),
        map_id: world.map_id(),
    };

    let data = bincode::serialize(&manifest, Infinite)?;

    let manifest_path = get_manifest_file();
    println!("{:?}", manifest_path.display());
    let mut manifest = File::create(manifest_path).map_err(SerialError::from)?;
    manifest.write(data.as_slice()).map_err(SerialError::from)?;
    Ok(())
}

pub fn load_manifest() -> SerialResult<SaveManifest> {
    let manifest_path = get_manifest_file();

    let mut data: Vec<u8> = Vec::new();
    let mut manifest_file = File::open(manifest_path)?;
    manifest_file.read_to_end(&mut data)?;
    let manifest = bincode::deserialize(&data)?;

    Ok(manifest)
}

pub fn init_paths() -> SerialResult<()> {
    fs::create_dir_all(get_save_directory()).map_err(SerialError::from)
}

/// Global save data not tied to any specific map.
#[derive(Serialize, Deserialize)]
pub struct SaveManifest {
    pub globals: GlobalFlags,
    pub map_id: MapId,
}

impl SaveManifest {
    pub fn new() -> Self {
        SaveManifest {
            globals: GlobalFlags::new(),
            map_id: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use testing::*;
    use world::traits::*;

    #[test]
    fn test_manifest() {
        init_paths().unwrap();

        let mut context = test_context_bounded(100, 100);
        let globals = context.state.world.flags().get_globals();
        let map_id = 101;
        context.state.world.set_map_id(map_id);

        save_manifest(&context.state.world).unwrap();
        context.state.world.set_player(None);

        let manifest = load_manifest().unwrap();

        assert_eq!(manifest.globals, globals);
        assert_eq!(manifest.map_id, map_id);
    }
}

