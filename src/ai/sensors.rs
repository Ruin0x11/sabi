use std::collections::HashMap;

use actor::Actor;
use world::World;
use ai::AiProp;

pub struct Sensor {
    pub callback: Box<Fn(&World, &Actor) -> bool>,
}

impl Sensor {
    pub fn new<F>(callback: F) -> Self
        where F: 'static + Fn(&World, &Actor) -> bool {
        Sensor {
            callback: Box::new(callback),
        }
    }
}

fn target_visible(world: &World, actor: &Actor) -> bool {
    actor.ai.target.borrow()
        .map_or(false, |id| {
            let target = world.actor(&id);
            actor.can_see(&target.get_pos())
        })
}

fn target_dead(world: &World, actor: &Actor) -> bool {
    actor.ai.target.borrow()
        .map_or(false, |id| {
            let target = world.actor(&id);
            target.is_dead()
        })
}


fn next_to_target(world: &World, actor: &Actor) -> bool {
    actor.ai.target.borrow()
        .map_or(false, |id| {
            let target = world.actor(&id);
            actor.get_pos().next_to(target.get_pos())
        })
}

fn has_target(_world: &World, actor: &Actor) -> bool {
    actor.ai.target.borrow().is_some()
}

fn health_low(_world: &World, actor: &Actor) -> bool {
    actor.hp() < 20
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
