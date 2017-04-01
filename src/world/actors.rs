use world::*;
impl World {
    /// Return an iterator over the currently loaded set of Actors in this
    /// world across all chunks.
    pub fn actors(&mut self) -> hash_map::Values<ActorId, Actor> {
        self.actors.values()
    }

    pub fn actor(&self, id: &ActorId) -> &Actor {
        self.actors.get(id).expect("Actor not found!")
    }

    pub fn actor_mut(&mut self, id: &ActorId) -> &mut Actor {
        self.actors.get_mut(id).expect("Actor not found!")
    }

    pub fn actor_at(&self, world_pos: WorldPosition) -> Option<&Actor> {
        match self.actor_ids_by_pos.get(&world_pos) {
            Some(id) => {
                assert!(self.actors.contains_key(id), "Coord -> id, id -> actor maps out of sync!");
                self.actors.get(id)
            }
            None       => None
        }
    }

    pub fn actor_at_mut(&mut self, world_pos: WorldPosition) -> Option<&mut Actor> {
        match self.actor_ids_by_pos.get_mut(&world_pos) {
            Some(id) => {
                assert!(self.actors.contains_key(id), "Coord -> id, id -> actor maps out of sync!");
                self.actors.get_mut(id)
            }
            None       => None
        }
    }

    pub fn pre_update_actor_pos(&mut self, pos_now: WorldPosition, pos_next: WorldPosition) {
        let id = self.actor_ids_by_pos.remove(&pos_now).unwrap();
        self.actor_ids_by_pos.insert(pos_next, id);
    }

    pub fn add_actor(&mut self, actor: Actor) {
        assert!(!self.actors.contains_key(&actor.get_id()), "Actor with same id already exists!");
        self.turn_order.add_actor(actor.get_id(), 0);
        self.actor_ids_by_pos.insert(actor.get_pos(), actor.get_id());
        debug!(self.logger, "add: Actor {:8}", actor.get_id());
        self.actors.insert(actor.get_id(), actor);
    }

    pub fn remove_actor(&mut self, id: &ActorId) -> Actor {
        let pos = self.actor(id).get_pos();
        self.turn_order.remove_actor(id);
        self.actor_ids_by_pos.remove(&pos);
        let actor = self.actors.remove(id);
        assert!(actor.is_some(), "Tried removing nonexistent actor from world!");
        actor.unwrap()
    }

    pub fn with_actor<F>(&mut self, id: &ActorId, mut callback: F)
        where F: FnMut(&mut World, &mut Actor) {
        let mut actor = self.actors.remove(id).expect("Actor not found!");
        callback(self, &mut actor);
        self.actors.insert(id.clone(), actor);
    }

    pub fn player(&self) -> &Actor {
        self.actors.get(&self.player_id()).unwrap()
    }

    pub fn player_id(&self) -> ActorId {
        self.player_id.unwrap()
    }

    pub fn set_player_id(&mut self, id: ActorId) {
        self.player_id = Some(id);
    }


    pub fn is_player(&self, id: &ActorId) -> bool {
        &self.player_id() == id
    }

    pub fn next_actor(&mut self) -> Option<ActorId> {
        self.turn_order.next()
    }
}
