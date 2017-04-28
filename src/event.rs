use actor::{Actor, ActorId};
use logic::Action;
use world::*;
use point::{PointArea, SquareArea};
use drawcalls::*;
use stats::properties::Prop::*;

pub use self::EventKind::*;

/// An event that can be broadcast to an area of a map. After actions are run,
/// things inside the range that need to respond to the message can do so.
pub struct Event {
    pub area: EventArea,
    pub kind: EventKind,
}

#[derive(Debug)]
pub enum EventKind {
    SayName,
    SayThing(String),
    Explosion,
}

pub enum EventArea {
    Square(i32, i32, i32),
    Actor(ActorId),
}

pub fn check_all(world: &World) -> Vec<(Action, ActorId)> {
    let mut reactions = Vec::new();
    for event in &world.events {
        let mut area_iter = get_event_area_iter(world, &event.area);
        while let Some(pos) = area_iter.next() {
            world.draw_calls.push(Draw::Point(pos.x, pos.y));

            if let Some(actor) = world.actor_at(pos) {
                if !actor.is_dead() {
                    debug!(world.logger, "{} not dead, handling event {:?}", actor, event.kind);
                    if let Some(action) = handle_event(actor, world, &event.kind) {
                        reactions.push((action, actor.get_id()))
                    }
                }
            }
        }
    }
    reactions
}

fn handle_event(actor: &Actor,
                _world: &World,
                event: &EventKind) -> Option<(Action)> {
    // Oh god.
    match *event {
        EventKind::Explosion => {
            if actor.properties.check_bool(Explosive) {
                Some(Action::Explod)
            } else {
                Some(Action::Hurt(10))
            }
        }
        _ => {
            warn!(actor.logger, "{} can't handle event {:?}!", actor, event);
            None
        }
    }
}

fn get_event_area_iter(world: &World, area: &EventArea) -> Box<WorldIter> {
    match *area {
        EventArea::Square(x, y, r) => Box::new(SquareArea::new(WorldPosition::new(x, y), r)),
        EventArea::Actor(id)       => Box::new(PointArea::new(world.actor(&id).get_pos())),

    }
}

// impl World {
//     pub fn run_next_event(&mut self, event: Event) -> Vec<(Action, ActorId)> {
//         if let Some(event) = self.event_queue.pop() {
//             for pos in event.iter {
//                 if let Some(actor) = self.actor_at_mut(pos) {
//                     say_thing(actor, self, &event);
//                 }
//             }
//         }
//     }
// }
