//! Various functions that would normally be methods on Entity, if
//! such a thing were possible. These are for making getting component
//! data from the ECS less unwieldy. Also, it doesn't make sense for
//! the world itself to calculate this data regarding a specific
//! entity for us.

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

pub fn name(entity: Entity, world: &EcsWorld) -> String {
    world.ecs().names.get(entity)
        .map_or("(unnamed)".to_string(), |n| n.name.clone())
}

pub fn is_dead(entity: Entity, world: &EcsWorld) -> bool {
    world.ecs().healths.get(entity)
        .map_or(true, |h| h.is_dead())
}

pub fn can_see_other(viewer: Entity, target: Entity, world: &EcsWorld) -> bool {
    if let Some(target_pos) = world.position(target) {
        if !world.is_player(viewer) {
            return has_los(viewer, target_pos, world);
        }

        let fov = world.ecs().fovs.get(viewer);

        fov.map_or(false, |v| v.is_visible(&target_pos))
    } else {
        false
    }
}
