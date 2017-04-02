use world::*;

impl World {
    /// Return an iterator over the currently loaded set of Actors in this
    /// world across all chunks.
    pub fn actors(&mut self) -> hash_map::Values<ActorId, Actor> {
        self.actors.values()
    }

    // FIXME: This should be okay to return just &Actor, because the only
    // invalid cases are dead actors, and by the time this is called they should
    // be cleaned up.
    pub fn actor(&self, id: &ActorId) -> Option<&Actor> {
        if self.killed_actors.contains(id) {
            debug!(self.logger, "{} is dead.", id);
            None
        } else {
            self.actors.get(id)
        }
    }

    pub fn actor_mut(&mut self, id: &ActorId) -> Option<&mut Actor> {
        if self.killed_actors.contains(id) {
            debug!(self.logger, "{} is dead.", id);
            None
        } else {
            self.actors.get_mut(id)
        }
    }

    /// Returns a copy of the ID of the actor at point.
    pub fn actor_id_at(&self, world_pos: WorldPosition) -> Option<ActorId> {
        self.actor_ids_by_pos.get(&world_pos).map(|i| i.clone())
    }

    /// Returns a reference to the actor at point.
    pub fn actor_at(&self, world_pos: WorldPosition) -> Option<&Actor> {
        match self.actor_id_at(world_pos) {
            Some(id) => {
                assert!(self.actors.contains_key(&id), "Coord -> id, id -> actor maps out of sync!");
                self.actors.get(&id)
            }
            None       => None
        }
    }

    /// Returns a mutable reference to the actor at point.
    pub fn actor_at_mut(&mut self, world_pos: WorldPosition) -> Option<&mut Actor> {
        match self.actor_ids_by_pos.get_mut(&world_pos) {
            Some(id) => {
                assert!(self.actors.contains_key(id), "Coord -> id, id -> actor maps out of sync!");
                self.actors.get_mut(id)
            }
            None       => None
        }
    }

    /// Updates the position of the actor at `pos_now`.
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
        debug!(self.logger, "removing {:8}", id);
        let pos = self.actor(id).expect("Can't find actor to remove").get_pos();
        self.turn_order.remove_actor(id);
        self.actor_ids_by_pos.remove(&pos);
        let actor = self.actors.remove(id);
        assert!(actor.is_some(), "Tried removing nonexistent actor from world!");
        actor.unwrap()
    }

    /// Wrapper to move an actor out of the world's actor hashmap, so it can be
    /// mutated, then putting it back into the hashmap after.
    pub fn with_moved_actor<F>(&mut self, id: &ActorId, mut callback: F)
        where F: FnMut(&mut World, &mut Actor) {
        assert!(!self.killed_actors.contains(id), "Actor {} is dead!", id);
        let mut actor = self.actors.remove(id).expect("Actor not found!");
        callback(self, &mut actor);
        self.actors.insert(id.clone(), actor);
    }

    pub fn player(&self) -> &Actor {
        self.actors.get(&self.player_id()).expect("Player not found!")
    }

    pub fn player_id(&self) -> ActorId {
        self.player_id.expect("No player in this world!")
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

    pub fn actor_killed(&mut self, id: ActorId) {
        debug!(self.logger, "Killing: {}", id);

        // FIXME: Since the player should be able to hang around after death,
        // this shouldn't be done.
        self.remove_actor(&id);

        self.killed_actors.insert(id);
    }

    pub fn was_killed(&self, id: &ActorId) -> bool {
        self.killed_actors.contains(id)
    }

    pub fn purge_dead(&mut self) {
        let dead_ids = self.actors.iter()
            .filter(|&(_, actor)| actor.is_dead())
            .map(|(k, _)| k).cloned().collect::<Vec<ActorId>>();
        for id in dead_ids {
            debug!(self.logger, "{} was killed, purging.", id);
            self.actor_killed(id);
        }
    }
}
