pub struct Spatial {
    actors: HashMap<ActorId, Actor>,
    actor_ids_by_pos: HashMap<WorldPosition, ActorId>,
    // Actors that were killed during the current actor's turn, by events, etc.
    killed_actors: HashMap<ActorId, Actor>,

    // TEMP: Move upward
    player_id: Option<ActorId>,
}
