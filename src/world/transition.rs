use ecs::Loadout;
use point::Point;
use world::serial;
use world::EcsWorld;
use world::traits::*;
use world::flags::GlobalFlags;

struct TransitionData {
    pub globals: GlobalFlags,

    pub player_data: Loadout,
}

impl Transition<TransitionData> for EcsWorld {
    fn map_id(&self) -> u32 {
        self.flags.map_id
    }

    fn set_map_id(&mut self, id: u32) {
        self.flags.map_id = id;
        self.terrain_mut().set_id(id);
    }

    fn get_transition_data(&mut self) -> TransitionResult<TransitionData> {
        let player = self.player().unwrap();
        let loadout = self.unload_entity(player);
        let data = TransitionData {
            globals: self.flags().get_globals(),

            player_data: loadout,
        };

        Ok(data)
    }

    fn inject_transition_data(&mut self, previous: TransitionData) -> TransitionResult<()> {
        let map_id = self.flags.map_id;

        // because EncodeRng doesn't implement clone
        self.flags_mut().globals = previous.globals;

        // TODO: shouldn't have to set manually.
        self.set_map_id(map_id);

        let player_id = self.spawn(&previous.player_data, Point::new(0, 0));
        self.set_player(Some(player_id));

        Ok(())
    }
}

impl EcsWorld {
    pub fn get_map(&self, id: u32) -> Option<EcsWorld> {
        serial::load_world(id).ok()
    }

    pub fn move_to_map(&mut self, other: EcsWorld, dest: Point) -> TransitionResult<()> {
        let data = self.get_transition_data()?;

        serial::save_world(self).unwrap();

        *self = other;

        self.inject_transition_data(data)?;

        let player = self.player().expect("Player didn't move to new map!");
        self.set_entity_location(player, dest);

        // self.flags.map_id = other.flags.map_id;
        Ok(())
    }
}
