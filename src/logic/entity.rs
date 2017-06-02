use calx_ecs::Entity;

use point::{Point, LineIter};
use world::traits::*;
use world::EcsWorld;

pub fn has_los(looker: Entity, target_pos: Point, world: &EcsWorld) -> bool {
    let looker_pos = match world.position(looker) {
        Some(p) => p,
        None => return false,
    };

    for pos in LineIter::new(looker_pos, target_pos) {
        if !world.light_passes_through(&pos) {
            return false;
        }
    }

    true
}
