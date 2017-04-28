use point::Point;
use calx_ecs::Entity;

#[derive(Serialize, Deserialize, Clone)]
pub struct Flags {
    pub camera: Point,
    pub player: Option<Entity>,

    pub map_id: u32,
    pub max_map_id: u32,
    pub seed: u32,
}

impl Flags {
    pub fn new(seed: u32) -> Flags {
        Flags {
            camera: Point::new(0, 0),
            player: None,

            map_id: 0,
            max_map_id: 0,
            seed: seed,
        }
    }

    // pub fn seed(&self) -> u32 { self.seed }
}
