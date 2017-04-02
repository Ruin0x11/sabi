use std::collections::BTreeMap;
use std::iter::Iterator;
use std::cmp;

use actor::ActorId;

use world::*;

pub struct TurnOrder {
    times_until_turn: BTreeMap<ActorId, i32>,
}

impl TurnOrder {
    pub fn new() -> Self {
        TurnOrder {
            times_until_turn: BTreeMap::new(),
        }
    }

    pub fn add_actor(&mut self, id: ActorId, time: i32) {
        self.times_until_turn.insert(id, time);
    }

    pub fn remove_actor(&mut self, id: &ActorId) {
        self.times_until_turn.remove(id)
            .expect("Actor not in turn order map");
    }

    pub fn advance_time_for(&mut self, id: &ActorId, diff: i32) {
        let time_until_turn = self.times_until_turn.get_mut(id)
            .expect("Tried advancing time of actor not in turn order");
        *time_until_turn -= diff;
    }

    pub fn add_delay_for(&mut self, id: &ActorId, diff: i32) {
        let time_until_turn = self.times_until_turn.get_mut(id)
            .expect("Tried delaying time of actor not in turn order");
        *time_until_turn = cmp::max(0, *time_until_turn);
        *time_until_turn += diff;
    }

    pub fn get_time_for(&self, id: &ActorId) -> &i32 {
        self.times_until_turn.get(id)
            .expect("Actor not in turn order map")
    }
}

impl Iterator for TurnOrder {
    type Item = ActorId;
    fn next(&mut self) -> Option<ActorId> {
        if self.times_until_turn.len() == 0 {
            return None;
        }

        self.times_until_turn.iter()
            .min_by_key(|a| a.1)
            .map(|(a, _)| *a)
    }
}

impl World {
    /// Update the time-to-action for every actor in the world.
    /// The actor with the lowest time-to-action is the next one to act.
    pub fn advance_time(&mut self, amount: i32) {
        for id in self.actors.keys() {
            self.turn_order.advance_time_for(id, amount);
        }

        // The player is the only actor we might want to advance time for after
        // dying, and that's only for a single turn so that control returns to
        // the player and the death check can run instead of looping infinitely.
        let pid = self.player_id();
        if self.was_killed(&pid) {
            self.turn_order.advance_time_for(&pid, amount);
        }

        info!(self.logger, "world time advanced by {}", amount);
    }

    pub fn add_delay_for(&mut self, id: &ActorId, amount: i32) {
        self.turn_order.add_delay_for(id, amount);
    }

    pub fn time_until_turn_for(&self, id: &ActorId) -> i32 {
        *self.turn_order.get_time_for(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use world::*;
    use tile;
    use point::Point;

    fn get_world() -> World {
        let mut world = World::generate(WorldType::Instanced(WorldPosition::new(32, 32)),
                            16, tile::WALL);
        world.draw_square(WorldPosition::new(15, 15), 10, tile::FLOOR);
        world
    }

    #[test]
    fn test_single_id() {
        let mut turn_order = TurnOrder::new();
        let actor = ActorId::new_v4();
        turn_order.add_actor(actor, 0);

        assert_eq!(turn_order.next().unwrap(), actor);

        turn_order.add_delay_for(&actor, 100);

        assert_eq!(turn_order.next().unwrap(), actor);
    }

    #[test]
    fn test_two_ids() {
        let mut turn_order = TurnOrder::new();
        let actor_a = ActorId::new_v4();
        let actor_b = ActorId::new_v4();
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
        world.add_actor(other);
        world.draw_square(Point::new(15, 15), 10, tile::FLOOR);
    }
}
