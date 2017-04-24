use std::collections::HashMap;

use calx_ecs::Entity;

use ai::AiProp;
use world::World;

pub struct Sensor {
    pub callback: Box<Fn(&World, &Entity) -> bool>,
}

impl Sensor {
    pub fn new<F>(callback: F) -> Self
        where F: 'static + Fn(&World, &Entity) -> bool {
        Sensor {
            callback: Box::new(callback),
        }
    }
}

fn target_visible(world: &World, entity: &Entity) -> bool {
    false
}

fn target_dead(world: &World, entity: &Entity) -> bool {
    true
}

fn next_to_target(world: &World, entity: &Entity) -> bool {
    false
}

fn has_target(_world: &World, entity: &Entity) -> bool {
    true
}

fn health_low(_world: &World, entity: &Entity) -> bool {
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
