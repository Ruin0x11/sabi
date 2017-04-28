use ecs::Loadout;
use point::Point;
use world::serial;
use world::flags::Flags;
use world::EcsWorld;
use world::traits::*;

pub type TransitionResult<T> = Result<T, ()>;
pub trait Transition<T> {
    fn map_id(&self) -> u32;
    fn get_transition_data(&mut self) -> TransitionResult<T>;
    fn inject_transition_data(&mut self, data: T) -> TransitionResult<()>;
}

struct TransitionData {
    pub flags: Flags,
    pub player: Loadout,
}

impl Transition<TransitionData> for EcsWorld {
    fn map_id(&self) -> u32 {
        self.flags.map_id
    }

    fn get_transition_data(&mut self) -> TransitionResult<TransitionData> {
        let player = self.player().unwrap();
        let loadout = Loadout::get(self.ecs(), player);
        let data = TransitionData {
            flags: self.flags.clone(),
            player: loadout,
        };
        Ok(data)
    }

    fn inject_transition_data(&mut self, previous: TransitionData) -> TransitionResult<()> {
        let map_id = self.flags.map_id;
        self.flags = previous.flags;

        // TODO: shouldn't have to set manually.
        self.flags.map_id = map_id;
        self.terrain_mut().set_id(map_id);

        let player_id = self.spawn(&previous.player, Point::new(0, 0));
        self.flags.player = Some(player_id);

        Ok(())
    }
}

impl EcsWorld {
    pub fn get_map(&self, id: u32) -> Option<EcsWorld> {
        serial::load_world(id).ok()
    }

    pub fn move_to_map(&mut self, other: EcsWorld) -> TransitionResult<()> {
        debug!(self.logger, "player: {:?}", self.player());
        let data = self.get_transition_data()?;

        serial::save_world(self).map_err(|_| ())?;

        *self = other;

        self.inject_transition_data(data)?;

        // self.flags.map_id = other.flags.map_id;
        debug!(self.logger, "player after: {:?}", self.player());
        Ok(())
    }
}
