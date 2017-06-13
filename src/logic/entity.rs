//! Various functions that would normally be methods on Entity, if
//! such a thing were possible. These are for making getting component
//! data from the ECS less unwieldy. Also, it doesn't make sense for
//! the world itself to calculate this data regarding a specific
//! entity for us.

use calx_ecs::Entity;

use point::{Point, LineIter};
use world::traits::*;
use world::World;

pub trait EntityQuery {
    fn has_los(&self, target_pos: Point, world: &World) -> bool;
    fn name(&self, world: &World) -> String;
    fn is_dead(&self, world: &World) -> bool;
    fn can_see_other(&self, target: Entity, world: &World) -> bool;
}

impl EntityQuery for Entity {
    fn has_los(&self, target_pos: Point, world: &World) -> bool {
        let my_pos = match world.position(*self) {
            Some(p) => p,
            None => return false,
        };

        for pos in LineIter::new(my_pos, target_pos) {
            if !world.light_passes_through(&pos) {
                return false;
            }
        }

        true
    }

    fn name(&self, world: &World) -> String {
        world.ecs().names.get(*self)
            .map_or("(unnamed)".to_string(), |n| n.name.clone())
    }

    fn is_dead(&self, world: &World) -> bool {
        world.ecs().healths.get(*self)
            .map_or(true, |h| h.is_dead())
    }

    fn can_see_other(&self, target: Entity, world: &World) -> bool {
        if let Some(target_pos) = world.position(target) {
            if !world.is_player(*self) {
                return self.has_los(target_pos, world);
            }

            let fov = world.ecs().fovs.get(*self);

            fov.map_or(false, |v| v.is_visible(&target_pos))
        } else {
            false
        }
    }

}
