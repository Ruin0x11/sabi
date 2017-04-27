use cell;
use chunk::{CHUNK_WIDTH, Chunk, ChunkIndex};
use world::WorldPosition;

use noise::{NoiseModule, Perlin, Seedable};

impl Chunk {
    pub fn gen_perlin(index: &ChunkIndex, seed: u32) -> Self {
        const COS_THETA: f32 = 0.99854;
        const SIN_THETA: f32 = 0.05408;
        const NOISE_SCALE: f32 = 0.05;
        const THRESHOLD: f32 = 0.30;

        let gen = Perlin::new().set_seed(seed as usize);

        let mut cells = Vec::new();
        let center = WorldPosition::from_chunk_index(*index);

        for j in 0..(CHUNK_WIDTH) {
            for i in 0..(CHUNK_WIDTH) {
                let ax = (center.x + i) as f32;
                let ay = (center.y + j) as f32;
                let az = 0.2333333333;

                // Perlin doesn't work on integer values, so rotate slightly.
                let conv = |a: f32, b| NOISE_SCALE * (a * COS_THETA + b * SIN_THETA);
                let res = gen.get([conv(ay, -ax), conv(ax, ay), az]);

                if res > THRESHOLD {
                    cells.push(cell::WALL);
                } else {
                    cells.push(cell::FLOOR);
                }
            }
        }

        Chunk {
            cells: cells
        }
    }
}
