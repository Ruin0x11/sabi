use slog::Logger;

use log;

use world::*;
use world::turn_order::TurnOrder;

pub struct Actors {
    // NOTE: could also implement by putting each in its own Chunk
    actors: HashMap<ActorId, Actor>,
    actor_ids_by_pos: HashMap<WorldPosition, ActorId>,
    // Actors that were killed during the current actor's turn, by events, etc.
    killed_actors: HashMap<ActorId, Actor>,

    logger: Logger,

    // NOTE: I'm not sure it makes sense for a player to be tied to an existing
    // world, but it works for now.
    player_id: Option<ActorId>,
    // NOTE: Also must keep track of following actors, to move them between
    // areas.
}

impl Actors {
    pub fn new() -> Self {
        Actors {
            actors: HashMap::new(),
            actor_ids_by_pos: HashMap::new(),
            killed_actors: HashMap::new(),
            player_id: None,
            logger: log::make_logger("actors").unwrap(),
        }
    }

    /// Return an iterator over the currently loaded set of living Actors in
    /// this world across all chunks.
    pub fn iter(&mut self) -> hash_map::Values<ActorId, Actor> {
        self.actors.values()
    }

    pub fn ids(&mut self) -> hash_map::Keys<ActorId, Actor> {
        self.actors.keys()
    }

    pub fn get(&self, id: &ActorId) -> &Actor {
        if self.was_killed(id) {
            self.killed_actors.get(id).expect("No such actor!")
        } else {
            self.actors.get(id).expect("No such actor!")
        }
    }

    pub fn get_mut(&mut self, id: &ActorId) -> &mut Actor {
        if self.was_killed(id) {
            self.killed_actors.get_mut(id).expect("No such actor!")
        } else {
            self.actors.get_mut(id).expect("No such actor!")
        }
    }

    /// Returns a copy of the ID of the actor at point.
    pub fn id_at_pos(&self, world_pos: WorldPosition) -> Option<ActorId> {
        self.actor_ids_by_pos.get(&world_pos).map(|i| i.clone())
    }

    /// Returns a reference to the actor at point.
    pub fn at_pos(&self, world_pos: WorldPosition) -> Option<&Actor> {
        match self.id_at_pos(world_pos) {
            Some(id) => {
                assert!(self.actors.contains_key(&id), "Coord -> id, id -> actor maps out of sync!");
                self.actors.get(&id)
            }
            None       => None
        }
    }

    /// Returns a mutable reference to the actor at point.
    pub fn at_pos_mut(&mut self, world_pos: WorldPosition) -> Option<&mut Actor> {
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

    pub fn add(&mut self, actor: Actor) {
        assert!(!self.actors.contains_key(&actor.get_id()), "Actor with same id already exists!");
        self.actor_ids_by_pos.insert(actor.get_pos(), actor.get_id());
        debug!(self.logger, "adding: {:8}", actor.get_id());
        self.actors.insert(actor.get_id(), actor);
    }

    pub fn remove(&mut self, id: &ActorId) -> Actor {
        let actor = self.actors.remove(id);
        assert!(actor.is_some(), "Tried removing nonexistent actor from world!");
        actor.unwrap()
    }

    pub fn actor_killed(&mut self, id: ActorId) {
        if self.was_killed(&id) {
            warn!(self.logger, "Attempt to kill twice! {}", id);
            return;
        }
        debug!(self.logger, "Killing: {}", id);

        let actor = self.remove(&id);
        self.killed_actors.insert(id, actor);
    }

    pub fn make_actor_inactive(&mut self, id: &ActorId) {
        debug!(self.logger, "removing: {:8}", id);
        let pos = self.get(id).get_pos();

        self.actor_ids_by_pos.remove(&pos);
    }

    pub fn was_killed(&self, id: &ActorId) -> bool {
        self.killed_actors.contains_key(id)
    }

    pub fn purge_dead(&mut self) {
        let dead_ids = self.actors.iter()
            .filter(|&(_, actor)| actor.is_dead())
            .map(|(id, _)| id).cloned().collect::<Vec<ActorId>>();

        for id in dead_ids {
            debug!(self.logger, "{} was killed, purging.", id);
            self.actor_killed(id);
        }
    }

    pub fn player(&self) -> &Actor {
        self.actors.get(&self.player_id()).expect("Player not found!")
    }

    pub fn player_id(&self) -> ActorId {
        self.player_id.expect("No player has been set!")
    }

    pub fn set_player_id(&mut self, id: ActorId) {
        self.player_id = Some(id);
    }

    pub fn is_player(&self, id: &ActorId) -> bool {
        self.player_id() == *id
    }

    pub fn remove_partial(&mut self, id: &ActorId) -> Option<Actor> {
        assert!(!self.killed_actors.contains_key(id), "Actor {} is dead!", id);
        self.actors.remove(id)
    }

    pub fn insert_partial(&mut self, actor: Actor) {
        self.actors.insert(actor.get_id(), actor);
    }
}
