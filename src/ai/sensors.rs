use std::collections::HashMap;

use calx_ecs::Entity;

use ecs::traits::*;
use logic::entity::EntityQuery;
use world::traits::Query;
use world::World;

use super::{Ai, AiFacts, Target, TargetKind};

macro_rules! generate_sensors {
    ( $( $prop:ident, $default:expr, $sensor:ident );+ $(;)*) => {
        macro_attr! {
        #[derive(Serialize, Deserialize, Hash, Ord, PartialOrd, Eq, PartialEq, Debug, Clone, EnumFromStr!)]
        pub enum AiProp {
            $(
                $prop,
            )*
        }
        }

        pub(super) fn default_ai_facts() -> AiFacts {
            let mut facts = AiFacts::new();
            $(
                facts.insert(AiProp::$prop, $default);
            )*;
            facts
        }


        pub(super) fn all_props() -> Vec<AiProp> {
            vec![
                $(
                    AiProp::$prop,
                )*
            ]
        }

        pub fn make_sensors() -> HashMap<AiProp, Sensor> {
            let mut results = HashMap::new();
            $(
                results.insert(AiProp::$prop, Sensor::new($sensor));
            )*;
            results
        }
    }
}

generate_sensors! {
    HasTarget, false, sense_has_target;
    HasAttackTarget, false, sense_has_attack_target;
    HasPickupTarget, false, sense_has_pickup_target;
    TargetVisible, false, sense_target_visible;
    TargetDead, false, sense_target_dead;
    NextToTarget, false, sense_next_to_target;

    HealthLow, false, sense_health_low;

    CanDoRanged, false, sense_always_false; //TODO
    CanDoMelee, false, sense_always_true; //TODO
    HasThrowable, false, sense_has_throwable;
    AtPosition, false, sense_at_position;
    TargetInInventory, false, sense_target_in_inventory;
    OnTopOfTarget, false, sense_on_top_of_target;
    HasHealing, false, sense_always_false;
    FoundItem, false, sense_found_item;
    HealingItemNearby, false, sense_always_false;
    ThrowableNearby, false, sense_throwable_nearby;

    TargetClose, false, sense_always_false;
    TargetInRange, false, sense_target_in_range;

    Exists, true, sense_always_true;
    Moving, false, sense_always_false;
}

fn sense_has_throwable(world: &World, entity: Entity, ai: &Ai) -> bool {
    entity.inventory(world)
          .iter()
          .any(|item| item.basename(world) == "watermelon")
}

fn sense_target_visible(world: &World, entity: Entity, ai: &Ai) -> bool {
    ai.data.targets.borrow().peek().map_or(false, |t| {
        let pos = match world.position(t.entity) {
            Some(t) => t,
            None => return false,
        };

        entity.has_los(pos, world, Some(6))
    })
}

fn sense_target_dead(world: &World, _entity: Entity, ai: &Ai) -> bool {
    ai.data
      .targets
      .borrow()
      .peek()
      .map_or(false, |t| !world.is_alive(t.entity))
}

fn sense_next_to_target(world: &World, entity: Entity, ai: &Ai) -> bool {
    ai.data.targets.borrow().peek().map_or(false, |t| {
        let pos = match world.position(t.entity) {
            Some(p) => p,
            None => return false,
        };

        world.position(entity).unwrap().is_next_to(pos)
    })
}

fn sense_on_top_of_target(world: &World, entity: Entity, ai: &Ai) -> bool {
    ai.data.targets.borrow().peek().map_or(false, |t| {
        let pos = match world.position(t.entity) {
            Some(p) => p,
            None => return false,
        };

        world.position(entity).map_or(false, |ep| ep == pos)
    })
}

fn sense_target_in_range(world: &World, entity: Entity, ai: &Ai) -> bool {
    ai.data.targets.borrow().peek().map_or(false, |t| {
        let pos = match world.position(t.entity) {
            Some(p) => p,
            None => return false,
        };

        pos.distance(world.position(entity).unwrap()) < 6.0
    })
}

fn sense_target_in_inventory(world: &World, entity: Entity, ai: &Ai) -> bool {
    ai.data.targets.borrow().peek().map_or(false, |t| {
        let e = world.entities_in(entity);
        debug_ecs!(world, entity, "CONT: {:?} {:?}", t, e);
        e.contains(&t.entity)
    })
}

fn sense_at_position(world: &World, entity: Entity, ai: &Ai) -> bool {
    ai.data
      .important_pos
      .borrow()
      .map_or(false, |pos| world.position(entity).map_or(false, |ep| ep == pos))
}

fn sense_has_target(_world: &World, _entity: Entity, ai: &Ai) -> bool {
    !ai.data.targets.borrow().is_empty()
}

fn sense_has_attack_target(_world: &World, _entity: Entity, ai: &Ai) -> bool {
    ai.data
      .targets
      .borrow()
      .peek()
      .map_or(false, |t| t.kind == TargetKind::Attack)
}

fn sense_has_pickup_target(_world: &World, _entity: Entity, ai: &Ai) -> bool {
    ai.data
      .targets
      .borrow()
      .peek()
      .map_or(false, |t| t.kind == TargetKind::Pickup)
}

fn sense_health_low(world: &World, entity: Entity, _ai: &Ai) -> bool {
    world.ecs()
         .healths
         .map_or(false, |h| h.percent() < 0.2, entity)
}

fn sense_found_item(world: &World, entity: Entity, _ai: &Ai) -> bool {
    world.seen_entities(entity)
         .iter()
         .any(|i| world.is_item(*i))
}

fn sense_throwable_nearby(world: &World, entity: Entity, ai: &Ai) -> bool {
    world.seen_entities(entity)
         .iter()
         .any(|i| world.is_item(*i) && i.basename(world) == "watermelon")
}

fn sense_always_true(_world: &World, _entity: Entity, _ai: &Ai) -> bool {
    true
}

fn sense_always_false(_world: &World, _entity: Entity, _ai: &Ai) -> bool {
    false
}


pub struct Sensor {
    pub callback: Box<Fn(&World, Entity, &Ai) -> bool>,
}

impl Sensor {
    pub fn new<F>(callback: F) -> Self
    where
        F: 'static + Fn(&World, Entity, &Ai) -> bool,
    {
        Sensor { callback: Box::new(callback) }
    }
}

trait Sense {
    fn sense(world: &World, entity: Entity, ai: &Ai) -> bool;
}
