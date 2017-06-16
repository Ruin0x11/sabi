use std::collections::HashMap;

use calx_ecs::Entity;

use ai::{Ai, AiProp};
use ecs::traits::*;
use logic::entity::EntityQuery;
use world::traits::Query;
use world::World;

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

fn target_visible(world: &World, entity: Entity, ai: &Ai) -> bool {
    ai.target.borrow().map_or(false, |t| {
        let pos = world.position(t).expect("Target didn't have position!");
        entity.has_los(pos, world)
    })
}

fn target_dead(world: &World, _entity: Entity, ai: &Ai) -> bool {
    ai.target.borrow().map_or(false, |t| !world.is_alive(t))
}

fn next_to_target(world: &World, entity: Entity, ai: &Ai) -> bool {
    ai.target.borrow().map_or(false, |t| {
        world.position(entity).unwrap().is_next_to(
            world.position(t).unwrap(),
        )
    })
}

fn has_target(_world: &World, _entity: Entity, ai: &Ai) -> bool {
    ai.target.borrow().is_some()
}

fn health_low(world: &World, entity: Entity, _ai: &Ai) -> bool {
    world.ecs().healths.map_or(
        false,
        |h| h.percent() < 0.2,
        entity,
    )
}

fn always_true(world: &World, entity: Entity, _ai: &Ai) -> bool {
    true
}

fn always_false(world: &World, entity: Entity, _ai: &Ai) -> bool {
    false
}

pub fn make_sensors() -> HashMap<AiProp, Sensor> {
    let mut results = HashMap::new();
    results.insert(AiProp::TargetVisible, Sensor::new(target_visible));
    results.insert(AiProp::HasTarget, Sensor::new(has_target));
    results.insert(AiProp::TargetDead, Sensor::new(target_dead));
    results.insert(AiProp::NextToTarget, Sensor::new(next_to_target));
    results.insert(AiProp::HealthLow, Sensor::new(health_low));
    results.insert(AiProp::Exists, Sensor::new(always_true));
    results.insert(AiProp::Moving, Sensor::new(always_false));
    results
}
