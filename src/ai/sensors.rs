use std::collections::HashMap;

use calx_ecs::Entity;

use ai::{Ai, AiProp};
use ecs::traits::Query;
use world::EcsWorld;

pub struct Sensor {
    pub callback: Box<Fn(&EcsWorld, &Entity, &Ai) -> bool>,
}

impl Sensor {
    pub fn new<F>(callback: F) -> Self
        where F: 'static + Fn(&EcsWorld, &Entity, &Ai) -> bool {
        Sensor {
            callback: Box::new(callback),
        }
    }
}

fn target_visible(world: &EcsWorld, entity: &Entity, ai: &Ai) -> bool {
    ai.target.borrow()
        .map_or(false, |t| {
            world.can_see(*entity, world.position(t).unwrap())
        })
}

fn target_dead(world: &EcsWorld, entity: &Entity, ai: &Ai) -> bool {
    ai.target.borrow()
        .map_or(false, |t| {
            !world.is_alive(t)
        })
}

fn next_to_target(world: &EcsWorld, entity: &Entity, ai: &Ai) -> bool {
    ai.target.borrow()
        .map_or(false, |t| {
            world.position(*entity).unwrap().is_next_to(world.position(t).unwrap())
        })
}

fn has_target(_world: &EcsWorld, entity: &Entity, ai: &Ai) -> bool {
    ai.target.borrow().is_some()
}

fn health_low(_world: &EcsWorld, entity: &Entity, ai: &Ai) -> bool {
    false
}

pub fn make_sensors() -> HashMap<AiProp, Sensor> {
    let mut results = HashMap::new();
    results.insert(AiProp::TargetVisible, Sensor::new(target_visible) );
    results.insert(AiProp::HasTarget,     Sensor::new(has_target) );
    results.insert(AiProp::TargetDead,    Sensor::new(target_dead) );
    results.insert(AiProp::NextToTarget,  Sensor::new(next_to_target) );
    results.insert(AiProp::HealthLow,     Sensor::new(health_low) );
    results
}
