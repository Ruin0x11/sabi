use actor::{Actor, ActorId};
use action::Action;
use world::*;
use point::{PointArea, SquareArea};
use drawcalls::*;

pub use self::EventKind::*;

/// An event that can be broadcast to an area of a map. After actions are run,
/// things inside the range that need to respond to the message can do so.
pub struct Event {
    pub area: EventArea,
    pub kind: EventKind,
}

pub enum EventKind {
    SayName,
    SayThing(String),
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
                if !world.is_player(&actor.get_id()) {
                    reactions.push((Action::Hurt, actor.get_id()))
                }
            }
        }
    }
    reactions
}

fn get_event_area_iter(world: &World, area: &EventArea) -> Box<WorldIter> {
    match *area {
        EventArea::Square(x, y, r) => Box::new(SquareArea::new(WorldPosition::new(x, y), r)),
        EventArea::Actor(id)       => Box::new(PointArea::new(world.actor(&id).get_pos())),
    }
}

fn say_thing(actor: &mut Actor, world: &mut World, event: &Event) {
    if let SayThing(ref s) = event.kind {
        world.message(s.clone());
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
