use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::PathBuf;

use bincode::{self, Infinite};

use infinigen::*;
use world::EcsWorld;

pub const SAVE_DIRECTORY: &'static str = "save";

pub fn get_save_directory(id: u32) -> PathBuf {
    PathBuf::from(format!("{}/{}/", SAVE_DIRECTORY, id))
}

fn get_world_savefile(id: u32) -> PathBuf {
    let mut save_dir = get_save_directory(id);
    save_dir.push("world.bin");
    save_dir
}

pub fn save_world(world: &mut EcsWorld) -> SerialResult<()> {
    let indices = world.terrain.chunk_indices();
    for index in indices.iter() {
        world.unload_chunk(index)?;
    }

    let data = bincode::serialize(&world, Infinite)?;
    let id = 0;

    let save_path = get_world_savefile(id);

    let mut savefile = File::create(save_path).map_err(SerialError::from)?;
    savefile.write(data.as_slice()).map_err(SerialError::from)?;
    Ok(())
}

pub fn load_world() -> SerialResult<EcsWorld> {
    let id = 0;

    fs::create_dir_all(get_save_directory(id))?;
    let save_path = get_world_savefile(id);

    let mut data: Vec<u8> = Vec::new();
    let mut savefile = File::open(save_path)?;
    savefile.read_to_end(&mut data)?;
    let world = bincode::deserialize(&data)?;

    Ok(world)
}

pub fn init_paths() -> SerialResult<()> {
    fs::create_dir_all(SAVE_DIRECTORY).map_err(SerialError::from)
}
