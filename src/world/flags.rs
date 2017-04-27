use point::Point;
use calx_ecs::Entity;

#[derive(Serialize, Deserialize)]
pub struct Flags {
    pub camera: Point,
    pub player: Option<Entity>,

    pub seed: u32,
}

impl Flags {
    pub fn new(seed: u32) -> Flags {
        Flags {
            camera: Point::new(0, 0),
            player: None,

            seed: seed,
        }
    }

    // pub fn seed(&self) -> u32 { self.seed }
}
