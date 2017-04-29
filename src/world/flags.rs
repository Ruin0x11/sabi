use calx_alg::EncodeRng;
use calx_ecs::Entity;
use rand::{Rng, SeedableRng, XorShiftRng};

use point::Point;

#[derive(Serialize, Deserialize, Clone)]
pub struct Flags {
    pub globals: GlobalFlags,

    pub camera: Point,
    pub map_id: u32,
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
    pub fn new(seed: u32) -> Flags {
        Flags {
            globals: GlobalFlags {
                player: None,
                max_map_id: 0,
            },
            camera: Point::new(0, 0),
            map_id: 0,

            seed: seed,
            rng: SeedableRng::from_seed([seed, seed, seed, seed]),
        }
    }

    pub fn seed(&self) -> u32 { self.seed }
    pub fn rng<'a>(&'a mut self) -> &'a mut Rng { &mut self.rng }

    pub fn get_globals(&self) -> GlobalFlags {
        self.globals.clone()
    }
}
