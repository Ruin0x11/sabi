use infinigen::ManagedChunk;

#[derive(Serialize, Deserialize)]
pub struct SerialChunk {
    pub i: i32,
}

impl ManagedChunk for SerialChunk {
    const SECTOR_SIZE: usize = 4096;

    const REGION_WIDTH: i32 = 16;
}
