use noise::{NoiseModule, Perlin, Seedable};

use graphics::cell::{self, Cell};
use chunk::{CHUNK_WIDTH, Chunk, ChunkIndex, ChunkPosition};
use world::WorldPosition;
use prefab;
use lua;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ChunkType {
    Blank,
    Fill(Cell),
    Perlin,
    Lua
}

use self::ChunkType::*;

impl ChunkType {
    pub fn generate(&self, index: &ChunkIndex, seed: u32) -> Chunk {
        match *self {
            Blank => generate_blank(cell::FLOOR),
            Fill(cell) => generate_blank(cell),
            Perlin => generate_perlin(index, seed),
            Lua => generate_from_prefab(),
        }
    }
}

fn generate_blank(cell: Cell) -> Chunk {
    let mut cells = Vec::new();

    for _ in 0..(CHUNK_WIDTH) {
        for _ in 0..(CHUNK_WIDTH) {
            cells.push(cell.clone());
        }
    }

    Chunk {
        cells: cells
    }
}

fn generate_perlin(index: &ChunkIndex, seed: u32) -> Chunk {
    const COS_THETA: f32 = 0.99854;
    const SIN_THETA: f32 = 0.05408;
    const NOISE_SCALE: f32 = 0.05;
    const THRESHOLD: f32 = 0.30;

    let gen = Perlin::new().set_seed(seed as usize);

    let mut cells = Vec::new();
    let center = WorldPosition::from(*index);

    for j in 0..CHUNK_WIDTH {
        for i in 0..CHUNK_WIDTH {
            let ax = (center.x + i) as f32;
            let ay = (center.y + j) as f32;
            let az = 0.2333333333;

            // Perlin doesn't work on integer values, so rotate slightly.
            let conv = |a: f32, b| NOISE_SCALE * (a * COS_THETA + b * SIN_THETA);
            let res = gen.get([conv(ay, -ax), conv(ax, ay), az]);

            if res < 0.1 {
                cells.push(cell::WALL);
            } else if res < 0.4 {
                cells.push(cell::SAND);
            } else if res < 0.7 {
                cells.push(cell::GRASS);
            } else {
                cells.push(cell::FLOOR);
            }
        }
    }

    Chunk {
        cells: cells
    }
}

fn generate_from_prefab() -> Chunk {
    match lua::with_mut(|l| prefab::map_from_prefab(l, "prefab")) {
        Ok(prefab) => {
            let mut chunk = generate_blank(cell::FLOOR);
            for (point, cell) in prefab.iter() {
                *chunk.cell_mut(ChunkPosition::from(point)) = *cell;
            }
            chunk
        },
        Err(e) => {
            println!("{:?}", e);
            generate_blank(cell::FLOOR)
        }
    }
}

