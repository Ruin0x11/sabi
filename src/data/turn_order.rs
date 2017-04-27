use std::collections::BTreeMap;
use std::iter::Iterator;
use std::cmp;

use calx_ecs::Entity;

#[derive(Debug, Serialize, Deserialize)]
pub struct TurnOrder {
    times_until_turn: BTreeMap<Entity, i32>,
    paused: BTreeMap<Entity, i32>,
}

impl TurnOrder {
    pub fn new() -> Self {
        TurnOrder {
            times_until_turn: BTreeMap::new(),
            paused: BTreeMap::new(),
        }
    }

    pub fn pause(&mut self, id: Entity) {
        assert!(self.times_until_turn.contains_key(&id),
                "Tried pausing actor not in turn order map");
        let time = self.times_until_turn.remove(&id).unwrap();
        self.paused.insert(id, time);
    }

    pub fn resume(&mut self, id: Entity) {
        assert!(self.paused.contains_key(&id),
                "Tried resuming actor that wasn't paused");
        let time = self.paused.remove(&id).unwrap();
        self.times_until_turn.insert(id, time);
    }

    pub fn contains(&self, id: Entity) -> bool {
        self.times_until_turn.contains_key(&id)
    }

    pub fn insert(&mut self, id: Entity, time: i32) {
        assert!(!self.times_until_turn.contains_key(&id),
                "Entity already exists in turn order!");
        self.times_until_turn.insert(id, time);
    }

    pub fn remove(&mut self, id: &Entity) {
        let res = self.times_until_turn.remove(id);
        if let None = res {
            //warn!("Tried removing actor not in turn order map");
        }
    }

    pub fn advance_time_for(&mut self, id: &Entity, diff: i32) {
        let time_until_turn = self.times_until_turn.get_mut(id)
            .expect("Tried advancing time of actor not in turn order");
        *time_until_turn -= diff;
    }

    pub fn add_delay_for(&mut self, id: &Entity, diff: i32) {
        let time_until_turn = self.times_until_turn.get_mut(id)
            .expect("Tried delaying time of actor not in turn order");
        *time_until_turn = cmp::max(0, *time_until_turn);
        *time_until_turn += diff;
    }

    pub fn get_time_for(&self, id: &Entity) -> i32 {
        *self.times_until_turn.get(id)
            .expect("Actor not in turn order map")
    }
}

impl Iterator for TurnOrder {
    type Item = Entity;
    fn next(&mut self) -> Option<Entity> {
        if self.times_until_turn.len() == 0 {
            return None;
        }

        self.times_until_turn.iter()
            .min_by_key(|a| a.1)
            .map(|(a, _)| *a)
    }
}

#[cfg(never)]
#[cfg(test)]
mod tests {
    use super::*;
    use world::*;
    use tile;
    use point::Point;

    fn get_world<'a>() -> World {
        let mut world = World::generate(WorldType::Instanced(WorldPosition::new(32, 32)),
                            16, tile::WALL);
        world.draw_square(WorldPosition::new(15, 15), 10, tile::FLOOR);
        world
    }

    #[test]
    fn test_single_id() {
        let mut turn_order = TurnOrder::new();
        let actor = Entity::new_v4();
        turn_order.add_actor(actor, 0);

        assert_eq!(turn_order.next().unwrap(), actor);

        turn_order.add_delay_for(&actor, 100);

        assert_eq!(turn_order.next().unwrap(), actor);
    }

    #[test]
    fn test_two_ids() {
        let mut turn_order = TurnOrder::new();
        let actor_a = Entity::new_v4();
        let actor_b = Entity::new_v4();
        turn_order.add_actor(actor_a, 0);
        turn_order.add_actor(actor_b, 10);

        assert_eq!(turn_order.next().unwrap(), actor_a);

        turn_order.add_delay_for(&actor_a, 100);
        assert_eq!(turn_order.next().unwrap(), actor_b);

        turn_order.add_delay_for(&actor_b, 100);
        assert_eq!(turn_order.next().unwrap(), actor_a);

        turn_order.advance_time_for(&actor_b, 100);
        assert_eq!(turn_order.next().unwrap(), actor_b);
    }

    use actor::*;
    use glyph::Glyph;

    #[test]
    fn test_two_actors() {
        let mut world = get_world();

        let mut player = Actor::new(6, 6, Glyph::Player);
        player.speed = 300;

        let mut other = Actor::new(10, 10, Glyph::Player);
        other.speed = 100;
        world.actors.add(other);
        world.draw_square(Point::new(15, 15), 10, tile::FLOOR);
    }
}
