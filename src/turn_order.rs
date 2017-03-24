use std::collections::BTreeMap;
use std::iter::Iterator;

use actor::ActorId;

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
            .expect("Tried advancing time of actor not in turn order");
        *time_until_turn += diff;
    }

    pub fn get_time_for(&mut self, id: ActorId) -> &i32 {
        self.times_until_turn.get(&id)
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
            .map(|(a, b)| *a)
    }
}
