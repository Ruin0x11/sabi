use graphics::cell::{Cell, StairDir};
use point::Point;

pub struct Prefab {
    markers: HashMap<Point, PrefabMarker>;
    terrain: Vec<Cell>,
    size: Point,
}

pub enum PrefabMarker {
    Mob(String),
    Door,
    Stairs(StairDir)
}
