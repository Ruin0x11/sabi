//! Various functions that would normally be methods on Entity, if
//! such a thing were possible. These are for making getting component
//! data from the ECS less unwieldy. Also, it doesn't make sense for
//! the world itself to calculate this data regarding a specific
//! entity for us.

use std::cmp::Ordering::Equal;
use calx_ecs::Entity;

use ai::{AiTrigger, Attitude};
use data::{Properties, GetProp, Walkability};
use ecs::components::Gender;
use ecs::traits::*;
use ecs::Loadout;
use ecs::party::*;
use point::{Direction, Point, LineIter};
use util::grammar::{self, VerbPerson};
use uuid;
use world::traits::*;
use world::World;

pub trait EntityQuery {
    fn has_los(&self, target_pos: Point, world: &World, limit: Option<usize>) -> bool;
    fn name(&self, world: &World) -> String;
    fn name_with_article(&self, world: &World) -> String;
    fn verb_person(&self, world: &World) -> VerbPerson;

    fn inventory(&self, world: &World) -> Vec<Entity>;
    fn closest_entity(&self, entities: Vec<Entity>, world: &World) -> Option<Entity>;
    fn get_prop<T: Clone>(&self, prop: &str, world: &World) -> Option<T>
    where
        Properties: GetProp<T>;
    fn prop_equals<T: Clone + Eq>(&self, key: &str, val: T, world: &World) -> bool
    where
        Properties: GetProp<T>;
    fn uuid(&self, world: &World) -> Option<uuid::Uuid>;
    fn attitude(&self, other: Entity, world: &World) -> Attitude;
    fn walkable_pos_nearby(&self, world: &World) -> Option<Point>;

    fn is_dead(&self, world: &World) -> bool;
    fn is_pet(&self, world: &World) -> bool;
    fn is_hostile(&self, other: Entity, world: &World) -> bool;
    fn is_friendly(&self, other: Entity, world: &World) -> bool;
    fn can_see_pos(&self, pos: Point, world: &World) -> bool;
    fn can_see_other(&self, target: Entity, world: &World) -> bool;
}

impl EntityQuery for Entity {
    fn has_los(&self, target_pos: Point, world: &World, limit: Option<usize>) -> bool {
        let my_pos = match world.position(*self) {
            Some(p) => p,
            None => return false,
        };

        let mut length = 0;
        for pos in LineIter::new(my_pos, target_pos) {
            if !world.light_passes_through(&pos) {
                return false;
            }
            length += 1;
            if limit.map_or(false, |l| length > l) {
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

    fn can_see_pos(&self, pos: Point, world: &World) -> bool {
        if self.prop_equals("omniscient", true, world) {
            return true;
        }

        let fov = world.ecs().fovs.get(*self);
        fov.map_or(false, |v| v.is_visible(&pos))
    }

    fn can_see_other(&self, target: Entity, world: &World) -> bool {
        if let Some(target_pos) = world.position(target) {
            if !world.is_player(*self) {
                if world.is_player(target) {
                    // enemies can always see the player
                    return true;
                }
            }

            self.can_see_pos(target_pos, world)
        } else {
            false
        }
    }

    fn inventory(&self, world: &World) -> Vec<Entity> {
        world.entities_in(*self)
    }

    fn closest_entity(&self, entities: Vec<Entity>, world: &World) -> Option<Entity> {
        let mut dists: Vec<(Entity, f32)> = entities.iter()
                                                    .map(|&i| {
            let my_pos = world.position(*self).unwrap();
            let item_pos = world.position(i).unwrap();
            (i, my_pos.distance(item_pos))
        })
                                                    .collect();

        dists.sort_by(|&(_, a), &(_, b)| a.partial_cmp(&b).unwrap_or(Equal));

        dists.first().map(|&(e, _)| e)
    }

    fn walkable_pos_nearby(&self, world: &World) -> Option<Point> {
        world.position(*self).and_then(|pos| {
            let walkable: Vec<Point> = Direction::iter8()
                .map(|&d| pos + d)
                .filter(|p| world.can_walk(*p, Walkability::MonstersBlocking))
                .collect();

            walkable.first().cloned()
        })
    }

    fn get_prop<T: Clone>(&self, key: &str, world: &World) -> Option<T>
    where
        Properties: GetProp<T>,
    {
        world.ecs()
             .props
             .map_or(None, |props| props.props.get::<T>(key).ok().cloned(), *self)
    }

    fn prop_equals<T: Clone + Eq>(&self, key: &str, val: T, world: &World) -> bool
    where
        Properties: GetProp<T>,
    {
        self.get_prop(key, world).map_or(false, |p| p == val)
    }

    fn is_pet(&self, world: &World) -> bool {
        world.flags().globals.party.contains_member(*self, world)
    }

    fn uuid(&self, world: &World) -> Option<uuid::Uuid> {
        world.ecs().uuids.get(*self).map(|u| u.uuid.clone())
    }

    fn attitude(&self, other: Entity, world: &World) -> Attitude {
        if self.is_pet(world) {
            if world.is_player(other) {
                return Attitude::Friendly;
            }

            // Pets feel the same way as the player
            return world.player()
                        .map(|p| p.attitude(other, world))
                        .unwrap_or(Attitude::Neutral);
        }

        if world.is_player(*self) {
            if world.is_npc(other) || other.is_pet(world) {
                return Attitude::Friendly;
            }
        }

        if world.is_npc(*self) {
            Attitude::Friendly;
        }

        Attitude::Hostile
    }

    fn is_hostile(&self, other: Entity, world: &World) -> bool {
        self.attitude(other, world) == Attitude::Hostile
    }

    fn is_friendly(&self, other: Entity, world: &World) -> bool {
        self.attitude(other, world) == Attitude::Friendly
    }
}

pub trait EntityMutate {
    fn add_memory(&self, trigger: AiTrigger, world: &mut World);
    fn set_prop<T>(&self, key: &str, val: T, world: &mut World)
    where
        Properties: GetProp<T>;
    fn on_death(&self, world: &mut World);
    fn force_drop_all_items(&self, world: &mut World);
}

impl EntityMutate for Entity {
    fn add_memory(&self, trigger: AiTrigger, world: &mut World) {
        world.ecs_mut()
             .ais
             .map_mut(|ai| ai.data.add_memory(trigger), *self);
    }

    fn set_prop<T>(&self, key: &str, val: T, world: &mut World)
    where
        Properties: GetProp<T>,
    {
        world.ecs_mut()
             .props
             .map_mut(|props| props.props.set::<T>(key, val).unwrap(), *self);
    }

    fn on_death(&self, world: &mut World) {
        // if this is a pet, mark status as "dead" and don't drop anything
        if self.is_pet(world) {
            let loadout = Loadout::get(world.ecs(), *self);
            let uuid = self.uuid(world).unwrap();
            world.flags_mut()
                 .globals
                 .party
                 .set_status(uuid, PartyMemberStatus::Dead(loadout));
            mes!(world, "You feel sad.");
            return;
        }

        self.force_drop_all_items(world);
    }

    fn force_drop_all_items(&self, world: &mut World) {
        let pos = world.position(*self).unwrap();

        let inv = self.inventory(world);
        if inv.len() == 1 {
            let first = inv.first().unwrap();
            mes!(world, "{} falls to the ground.", first.name(world));
        } else if !inv.is_empty() {
            mes!(world, "Several items fall to the ground.");
        }
        for item in self.inventory(world) {
            world.place_entity(item, pos);
        }
    }
}
