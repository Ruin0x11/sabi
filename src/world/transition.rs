use std::path::Path;

use infinigen::*;

use ecs::Loadout;
use point::Point;
use world::serial;
use world::{World, MapId};
use world::traits::*;
use world::flags::GlobalFlags;

struct TransitionData {
    pub globals: GlobalFlags,

    pub player_data: Loadout,
}

impl Transition<TransitionData> for World {
    fn map_id(&self) -> MapId {
        self.flags.map_id
    }

    fn set_map_id(&mut self, id: MapId) {
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

        self.flags_mut().globals.player = previous.globals.player;
        let max_map_id = self.flags_mut().globals.max_map_id;
        if previous.globals.max_map_id > max_map_id {
            self.flags_mut().globals.max_map_id = previous.globals.max_map_id;
        }

        // TODO: shouldn't have to set manually.
        self.set_map_id(map_id);

        let player_id = self.spawn(&previous.player_data, Point::new(0, 0));
        self.set_player(Some(player_id));

        Ok(())
    }
}

impl World {
    pub fn move_to_map(&mut self, other: World, dest: Point) -> TransitionResult<()> {
        let data = self.get_transition_data()?;

        serial::save_world(self).unwrap();

        *self = other;

        self.inject_transition_data(data)?;

        let player = self.player().expect("Player didn't move to new map!");
        self.set_entity_location(player, dest);

        self.on_load();

        println!("id: {}", self.flags.map_id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecs;
    use graphics::cell::CellType;
    use point::POINT_ZERO;
    use testing::*;
    use world;
    use world::*;

    #[test]
    fn test_modify_before_transition() {
        let mut context = test_context_bounded(128, 128);

        let mut new_world = World::new()
            .from_other_world(&context.state.world)
            .with_bounds(Bounds::Bounded(64, 64))
            .build();

        let change_pos = POINT_ZERO;
        {
            let cell_mut = new_world.cell_mut(&change_pos);
            cell_mut.unwrap().type_ = CellType::Wall;
        }

        context.state.world.move_to_map(new_world, change_pos).unwrap();

        let cell = context.state.world.terrain().cell(&change_pos);
        assert!(cell.is_some(), "World terrain wasn't loaded in after transition");
        assert_eq!(cell.unwrap().type_, CellType::Wall);
    }

    #[test]
    fn test_modify_after_transition() {
        let mut context = test_context_bounded(128, 128);

        let new_world = World::new()
            .from_other_world(&context.state.world)
            .with_bounds(Bounds::Bounded(64, 64))
            .build();

        let change_pos = POINT_ZERO;

        context.state.world.move_to_map(new_world, change_pos).unwrap();

        let e = context.state.world.create(ecs::prefab::item("cola", "cola"), change_pos);

        assert!(context.state.world.position(e).is_some());
    }
    #[test]
    fn test_max_map_id() {
        let mut context = test_context_bounded(64, 64);
        let new_world = World::new()
            .from_other_world(&context.state.world)
            .build();
        assert_eq!(new_world.flags().globals.max_map_id, 1);

        assert_eq!(context.state.world.flags().globals.max_map_id, 0);

        let prev_id = context.state.world.flags().map_id;

        context.state.world.move_to_map(new_world, POINT_ZERO).unwrap();
        assert_eq!(context.state.world.flags().globals.max_map_id, 1);

        let prev_world = world::serial::load_world(prev_id).unwrap();

        context.state.world.move_to_map(prev_world, POINT_ZERO).unwrap();
        assert_eq!(context.state.world.flags().globals.max_map_id, 1);
    }
}
