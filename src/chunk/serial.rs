use chunk::Chunk;

use infinigen::ManagedChunk;

#[derive(Serialize, Deserialize)]
pub struct SerialChunk {
    pub chunk: Chunk,
}

impl ManagedChunk for SerialChunk {
    const SECTOR_SIZE: usize = 4096;
    const REGION_WIDTH: i32 = 16;
}
