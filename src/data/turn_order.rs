use std::collections::BTreeMap;
use std::iter::Iterator;
use std::cmp;

use calx_ecs::Entity;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TurnOrderError {
    NoSuchEntity,
    EntityPaused,
    EntityActive,
    AlreadyExists,
}

use self::TurnOrderError::*;

pub type TurnOrderResult<T> = Result<T, TurnOrderError>;

// NOTE: This could be implemented with priority queues, but whatever.
#[derive(Debug, Serialize, Deserialize)]
pub struct TurnOrder {
    active: BTreeMap<Entity, i32>,
    paused: BTreeMap<Entity, i32>,
}

impl TurnOrder {
    pub fn new() -> Self {
        TurnOrder {
            active: BTreeMap::new(),
            paused: BTreeMap::new(),
        }
    }

    pub fn pause(&mut self, id: Entity) -> TurnOrderResult<()> {
        if self.paused.contains_key(&id) {
            return Err(EntityPaused)
        }
        if !self.active.contains_key(&id) {
            return Err(NoSuchEntity);
        }
        let time = self.active.remove(&id).unwrap();
        self.paused.insert(id, time);
        Ok(())
    }

    pub fn resume(&mut self, id: Entity) -> TurnOrderResult<()> {
        if self.active.contains_key(&id) {
            return Err(EntityActive)
        }
        if !self.paused.contains_key(&id) {
            return Err(NoSuchEntity)
        }

        let time = self.paused.remove(&id).unwrap();
        self.active.insert(id, time);
        Ok(())
    }

    pub fn paused_contains(&self, id: Entity) -> bool {
        self.paused.contains_key(&id)
    }

    pub fn contains(&self, id: Entity) -> bool {
        self.active.contains_key(&id)
    }

    pub fn insert(&mut self, id: Entity, time: i32) -> TurnOrderResult<()> {
        if self.paused.contains_key(&id) || self.active.contains_key(&id) {
            return Err(AlreadyExists)
        }

        self.active.insert(id, time);
        Ok(())
    }

    pub fn remove(&mut self, id: Entity) -> TurnOrderResult<()> {
        let res = self.active.remove(&id);
        if res.is_none() {
            let paused = self.paused.remove(&id);
            if paused.is_none() {
                return Err(NoSuchEntity);
            }
        }
        Ok(())
    }

    pub fn advance_time_for(&mut self, id: Entity, diff: i32) -> TurnOrderResult<()> {
        if self.paused_contains(id) {
            return Err(EntityPaused);
        }

        let time_until_turn = match self.active.get_mut(&id) {
            Some(time) => time,
            None       => return Err(NoSuchEntity)
        };

        *time_until_turn -= diff;
        Ok(())
    }

    pub fn add_delay_for(&mut self, id: Entity, diff: i32) -> TurnOrderResult<()> {
        if self.paused_contains(id) {
            return Err(EntityPaused);
        }

        let time_until_turn = match self.active.get_mut(&id) {
            Some(time) => time,
            None       => return Err(NoSuchEntity)
        };

        *time_until_turn = cmp::max(0, *time_until_turn);
        *time_until_turn += diff;
        Ok(())
    }

    pub fn get_time_for(&self, id: Entity) -> TurnOrderResult<i32> {
        if self.paused_contains(id) {
            return Err(EntityPaused);
        }

        match self.active.get(&id) {
            Some(time) => Ok(*time),
            None       => Err(NoSuchEntity)
        }
    }
}

impl Iterator for TurnOrder {
    type Item = Entity;
    fn next(&mut self) -> Option<Entity> {
        if self.active.is_empty() {
            return None;
        }

        self.active.iter()
            .min_by_key(|a| a.1)
            .map(|(a, _)| *a)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecs::Ecs;

    #[test]
    fn test_single_id() {
        let mut ecs = Ecs::new();
        let mut turn_order = TurnOrder::new();
        let entity = ecs.make();
        turn_order.insert(entity, 0).unwrap();

        assert_eq!(turn_order.next(), Some(entity));

        turn_order.add_delay_for(entity, 100).unwrap();

        assert_eq!(turn_order.next(), Some(entity));
    }

    #[test]
    fn test_two_ids() {
        let mut ecs = Ecs::new();
        let mut turn_order = TurnOrder::new();

        let first_entity = ecs.make();
        let second_entity = ecs.make();
        turn_order.insert(first_entity, 0).unwrap();
        turn_order.insert(second_entity, 10).unwrap();

        assert_eq!(turn_order.next(), Some(first_entity));

        turn_order.add_delay_for(first_entity, 100).unwrap();
        assert_eq!(turn_order.next(), Some(second_entity));

        turn_order.add_delay_for(second_entity, 100).unwrap();
        assert_eq!(turn_order.next(), Some(first_entity));

        turn_order.advance_time_for(second_entity, 100).unwrap();
        assert_eq!(turn_order.next(), Some(second_entity));
    }

    #[test]
    fn test_pause_resume() {
        let mut ecs = Ecs::new();
        let mut turn_order = TurnOrder::new();

        let fast = ecs.make();
        let slow = ecs.make();
        turn_order.insert(fast, 1).unwrap();
        turn_order.insert(slow, 10000).unwrap();

        assert_eq!(turn_order.next(), Some(fast));

        turn_order.pause(fast).unwrap();
        assert_eq!(turn_order.next(), Some(slow));
        assert_eq!(turn_order.get_time_for(fast), Err(EntityPaused));
        assert_eq!(turn_order.add_delay_for(fast, 100), Err(EntityPaused));

        turn_order.resume(fast).unwrap();
        assert_eq!(turn_order.next(), Some(fast));
        assert_eq!(turn_order.get_time_for(fast), Ok(1));
        assert_eq!(turn_order.add_delay_for(fast, 100), Ok(()));
    }

    #[test]
    fn test_insert() {
        let mut ecs = Ecs::new();
        let mut turn_order = TurnOrder::new();

        let entity = ecs.make();

        assert_eq!(turn_order.insert(entity, 1), Ok(()));
        assert_eq!(turn_order.insert(entity, 100), Err(AlreadyExists));

        turn_order.pause(entity).unwrap();
        assert_eq!(turn_order.insert(entity, 100), Err(AlreadyExists));
    }

    #[test]
    fn test_remove() {
        let mut ecs = Ecs::new();
        let mut turn_order = TurnOrder::new();

        let entity = ecs.make();
        turn_order.insert(entity, 1).unwrap();

        assert_eq!(turn_order.remove(entity), Ok(()));
        assert_eq!(turn_order.remove(entity), Err(NoSuchEntity));

        turn_order.insert(entity, 1).unwrap();
        turn_order.pause(entity).unwrap();
        assert_eq!(turn_order.remove(entity), Ok(()));
        assert_eq!(turn_order.remove(entity), Err(NoSuchEntity));
    }

    #[test]
    fn test_pause_resume_twice() {
        let mut ecs = Ecs::new();
        let mut turn_order = TurnOrder::new();

        let fast = ecs.make();
        let slow = ecs.make();

        assert_eq!(turn_order.pause(fast), Err(NoSuchEntity));
        assert_eq!(turn_order.resume(fast), Err(NoSuchEntity));

        turn_order.insert(fast, 1).unwrap();
        turn_order.insert(slow, 10000).unwrap();

        assert_eq!(turn_order.resume(fast), Err(EntityActive));

        assert_eq!(turn_order.pause(fast), Ok(()));
        assert_eq!(turn_order.pause(fast), Err(EntityPaused));

        assert_eq!(turn_order.resume(fast), Ok(()));
        assert_eq!(turn_order.resume(fast), Err(EntityActive));
    }

}
