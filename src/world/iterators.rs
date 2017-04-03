use world::*;

pub struct ActorArea {
    positions: Vec<WorldPosition>
}

// NOTE: It could be possible to just iterate over actors.
impl ActorArea {
    pub fn new(world: &World, ids: Vec<ActorId>) -> Self {
        let mut positions = Vec::new();
        for id in ids {
            positions.push(world.actor(&id).get_pos());
        }
        ActorArea {
            positions: positions,
        }
    }
}

impl Iterator for ActorArea {
    type Item = WorldPosition;

    fn next(&mut self) -> Option<WorldPosition> {
        self.positions.pop()
    }
}
