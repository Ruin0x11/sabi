//! Various functions that would normally be methods on Entity, if
//! such a thing were possible. These are for making getting component
//! data from the ECS less unwieldy. Also, it doesn't make sense for
//! the world itself to calculate this data regarding a specific
//! entity for us.

use std::cmp::Ordering::Equal;
use calx_ecs::Entity;

use ai::AiTrigger;
use ecs::components::Gender;
use ecs::traits::*;
use point::{Point, LineIter};
use util::grammar::{self, VerbPerson};
use world::traits::*;
use world::World;

pub trait EntityQuery {
    fn has_los(&self, target_pos: Point, world: &World) -> bool;
    fn name(&self, world: &World) -> String;
    fn name_with_article(&self, world: &World) -> String;
    fn verb_person(&self, world: &World) -> VerbPerson;
    fn is_dead(&self, world: &World) -> bool;
    fn can_see_other(&self, target: Entity, world: &World) -> bool;
    fn inventory(&self, world: &World) -> Vec<Entity>;
    fn closest_entity(&self, entities: Vec<Entity>, world: &World) -> Option<Entity>;
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
        if world.is_player(*self) {
            return "you".to_string();
        }

        let name_compo = match world.ecs().names.get(*self) {
            Some(n) => n,
            None => return "something".to_string(),
        };

        if let Some(ref proper) = name_compo.proper_name {
            return format!("{} the {}", proper, name_compo.name);
        }

        format!("the {}", name_compo.name)
    }

    fn name_with_article(&self, world: &World) -> String {
        if world.is_player(*self) {
            return "yourself".to_string();
        }

        let name_compo = match world.ecs().names.get(*self) {
            Some(n) => n,
            None => return "something".to_string(),
        };

        if let Some(ref proper) = name_compo.proper_name {
            return format!("{} the {}", proper, name_compo.name);
        }

        format!("{} {}", grammar::get_article(&name_compo.name), name_compo.name)
    }

    fn verb_person(&self, world: &World) -> VerbPerson {
        let gender = world.ecs()
            .names
            .get(*self)
            .map_or(Gender::Unknown, |n| n.gender);
        if world.is_player(*self) {
            VerbPerson::You
        } else {
            match gender {
                Gender::Male => VerbPerson::He,
                Gender::Female => VerbPerson::She,
                _ => VerbPerson::It,
            }
        }
    }

    fn is_dead(&self, world: &World) -> bool {
        world.ecs().healths.get(*self).map_or(true, |h| h.is_dead())
    }

    fn can_see_other(&self, target: Entity, world: &World) -> bool {
        if let Some(target_pos) = world.position(target) {
            if !world.is_player(*self) {
                if world.is_player(target) {
                    // enemies can always see the player
                    return true;
                } else {
                    return self.has_los(target_pos, world);
                }
            }

            let fov = world.ecs().fovs.get(*self);

            fov.map_or(false, |v| v.is_visible(&target_pos))
        } else {
            false
        }
    }

    fn inventory(&self, world: &World) -> Vec<Entity> {
        world.entities_in(*self)
    }

    fn closest_entity(&self, entities: Vec<Entity>, world: &World) -> Option<Entity> {
        let mut dists: Vec<(Entity, f32)> = entities.iter().map(|&i| {
            let my_pos = world.position(*self).unwrap();
            let item_pos = world.position(i).unwrap();
            (i, my_pos.distance(item_pos))
        }).collect();

        dists.sort_by(|&(_, a), &(_, b)| a.partial_cmp(&b).unwrap_or(Equal));

        dists.first().map(|&(e, _)| e)
    }
}

pub trait EntityMutate {
    fn add_memory(&self, trigger: AiTrigger, world: &mut World);
    fn on_death(&self, world: &mut World);
}

impl EntityMutate for Entity {
    fn add_memory(&self, trigger: AiTrigger, world: &mut World) {
        world.ecs_mut().ais.map_mut(|ai| ai.add_memory(trigger), *self);
    }

    fn on_death(&self, world: &mut World) {
        let pos = world.position(*self).unwrap();

        let inv = self.inventory(world);
        if inv.len() == 1 {
            let first = inv.first().unwrap();
            mes!(world, "{} falls to the ground.", a=first.name(world));
        } else if !inv.is_empty() {
            mes!(world, "Several items fall to the ground.");
        }
        for item in self.inventory(world) {
            world.place_entity(item, pos);
        }
    }
}
