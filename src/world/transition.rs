use calx_ecs::Entity;
use infinigen::*;

use ecs::Loadout;
use point::Point;
use world::serial;
use world::{World, MapId};
use world::traits::*;
use world::flags::GlobalFlags;

#[derive(Debug)]
struct TransitionLoadout {
    parent: Loadout,
    children: Vec<TransitionLoadout>,
}

impl TransitionLoadout {
    fn from_entity(entity: Entity, world: &mut World) -> Self {
        // TODO: Does not handle recursive children
        let children = world.entities_in(entity).into_iter()
            .map(|e| TransitionLoadout {
                parent: world.unload_entity(e),
                children: Vec::new(),
            })
            .collect();
        let parent = world.unload_entity(entity);

        TransitionLoadout {
            parent: parent,
            children: children,
        }
    }

    fn inject(self, world: &mut World) -> Entity {
        let parent = world.spawn(&self.parent, Point::new(0, 0));
        for child in self.children.into_iter() {
            let child_entity = child.parent.make(world.ecs_mut());
            world.place_entity_in(parent, child_entity);
        }

        parent
    }
}

struct TransitionData {
    pub globals: GlobalFlags,

    pub player_data: TransitionLoadout
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
        let loadout = TransitionLoadout::from_entity(player, self);
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

        let player_id = previous.player_data.inject(self);
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
        self.place_entity(player, dest);

        self.on_load();

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecs;
    use logic::Action;
    use point::POINT_ZERO;
    use state;
    use testing::*;
    use world::*;
    use world;

    #[test]
    fn test_modify_before_transition() {
        let mut context = test_context_bounded(128, 128);

        let mut new_world = World::new()
            .from_other_world(&context.state.world)
            .with_bounds(Bounds::Bounded(64, 64))
            .build()
            .unwrap();

        let change_pos = POINT_ZERO;
        {
            let cell_mut = new_world.cell_mut(&change_pos);
            cell_mut.unwrap().set("wall");
        }

        context.state.world.move_to_map(new_world, change_pos).unwrap();

        let cell = context.state.world.terrain().cell(&change_pos);
        assert!(cell.is_some(), "World terrain wasn't loaded in after transition");
        assert_eq!(cell.unwrap().name(), "wall");
    }

    #[test]
    fn test_modify_after_transition() {
        let mut context = test_context_bounded(128, 128);

        let new_world = World::new()
            .from_other_world(&context.state.world)
            .with_bounds(Bounds::Bounded(64, 64))
            .build()
            .unwrap();

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
            .build()
            .unwrap();
        assert_eq!(new_world.flags().globals.max_map_id, 1);

        assert_eq!(context.state.world.flags().globals.max_map_id, 0);

        let prev_id = context.state.world.flags().map_id;

        context.state.world.move_to_map(new_world, POINT_ZERO).unwrap();
        assert_eq!(context.state.world.flags().globals.max_map_id, 1);

        let prev_world = world::serial::load_world(prev_id).unwrap();

        context.state.world.move_to_map(prev_world, POINT_ZERO).unwrap();
        assert_eq!(context.state.world.flags().globals.max_map_id, 1);
    }

    #[test]
    fn test_transition_loadout() {
        let mut context = test_context_bounded(64, 64);

        let item = context.state.world.create(ecs::prefab::item("cola", "cola"), POINT_ZERO);
        state::run_action(&mut context, Action::Pickup(item));

        let player = context.state.world.player().unwrap();
        assert_eq!(context.state.world.entities_in(player).len(), 1);

        let loadout = TransitionLoadout::from_entity(player, &mut context.state.world);
        assert_eq!(loadout.children.len(), 1);
    }

    #[test]
    fn test_inject_inventory() {
        let mut context = test_context_bounded(64, 64);
        let new_world = World::new()
            .from_other_world(&context.state.world)
            .build()
            .unwrap();

        let item = context.state.world.create(ecs::prefab::item("cola", "cola"), POINT_ZERO);
        state::run_action(&mut context, Action::Pickup(item));

        context.state.world.move_to_map(new_world, POINT_ZERO).unwrap();

        let player = context.state.world.player().unwrap();

        // Note that the entity has been removed and re-injected, so it isn't possible to compare
        // by ID.
        assert_eq!(context.state.world.entities_in(player).len(), 1);
    }
}
