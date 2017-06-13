use std::collections::HashSet;

use point::{CircleIter, LineIter, Point};
use infinigen::ChunkedWorld;
use terrain::traits::*;
use world::traits::*;
use world::World;

pub fn bresenham_fast(world: &World, center: Point, radius: i32) -> HashSet<Point> {
    let in_bounds = |pos| world.terrain().in_bounds(&pos);
    let light_passes = |pos| world.cell_const(&pos).map_or(false, |c| c.can_see_through());

    let mut visited = HashSet::new();
    let mut visible = HashSet::new();
    visible.insert(center);

    for point in CircleIter::new(center, radius) {
        let line = LineIter::new(center, point);
        for line_pos in line {
            if !in_bounds(line_pos) {
                break;
            }

            if !visited.contains(&line_pos) {
                visible.insert(line_pos);
                visited.insert(line_pos);
            }

            if !light_passes(line_pos) {
                break;
            }
        }
    }

    visible
}
