use std::collections::HashSet;

use calx_alg::EncodeRng;
use calx_ecs::Entity;
use rand::{SeedableRng, XorShiftRng};
use world::MapId;

use point::Point;

#[derive(Serialize, Deserialize)]
pub struct Flags {
    pub globals: GlobalFlags,

    pub camera: Point,
    pub map_id: MapId,
    pub explored: HashSet<Point>,
    seed: u32,
    rng: EncodeRng<XorShiftRng>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct GlobalFlags {
    pub max_map_id: u32,
    pub player: Option<Entity>,
}

impl GlobalFlags {
    pub fn new() -> Self {
        GlobalFlags {
            max_map_id: 0,
            player: None,
        }
    }
}

impl Flags {
    pub fn new(seed: u32, map_id: MapId) -> Flags {
        Flags {
            globals: GlobalFlags {
                player: None,
                max_map_id: map_id,
            },
            camera: Point::new(0, 0),
            map_id: map_id,
            explored: HashSet::new(),

            seed: seed,
            rng: SeedableRng::from_seed([seed, seed, seed, seed]),
        }
    }

    pub fn seed(&self) -> u32 { self.seed }
    pub fn rng(&mut self) -> &mut EncodeRng<XorShiftRng> { &mut self.rng }

    pub fn get_globals(&self) -> GlobalFlags {
        self.globals.clone()
    }
}
