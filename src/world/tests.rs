use logic::Action;
use state;
use testing::*;
use world::*;

#[test]
fn test_persistence() {
    let mut context = test_context_bounded(64, 64);

    let mob = {
        let world_mut = &mut context.state.world;
        place_mob(world_mut, WorldPosition::new(1, 1))
    };

    let world = &context.state.world;

    assert_eq!(is_persistent(world, world.player().unwrap()), true);
    assert_eq!(is_persistent(world, mob), false);
}


#[test]
fn test_alive_active() {
    let mut context = test_context_bounded(64, 64);
    let mob_pos = WorldPosition::new(1, 1);
    let mob_chunk = ChunkIndex::from_world_pos(mob_pos);

    let mob = {
        let world_mut = &mut context.state.world;
        place_mob(world_mut, mob_pos)
    };

    {
        let world = &context.state.world;
        assert_eq!(world.is_alive(mob), true);
        assert_eq!(world.is_active(mob), true);
        assert_eq!(world.ecs().contains(mob), true);
    }

    context.state.world.unload_chunk(&mob_chunk).unwrap();

    {
        let world = &context.state.world;
        assert_eq!(world.is_alive(mob), true);
        assert_eq!(world.is_active(mob), false);
        assert_eq!(world.ecs().contains(mob), true);
    }

    context.state.world.load_chunk(&mob_chunk).unwrap();
    context.state.world.kill(mob);

    {
        let world = &context.state.world;
        assert_eq!(world.is_alive(mob), false);
        assert_eq!(world.is_active(mob), true);
        assert_eq!(world.ecs().contains(mob), true);
    }

    context.state.world.update_killed();

    {
        let world = &context.state.world;
        assert_eq!(world.is_alive(mob), false);
        assert_eq!(world.is_active(mob), false);
        assert_eq!(world.ecs().contains(mob), true);
    }

    context.state.world.purge_dead();

    {
        let world = &context.state.world;
        assert_eq!(world.is_alive(mob), false);
        assert_eq!(world.is_active(mob), false);
        assert_eq!(world.ecs().contains(mob), false);
    }
}

#[test]
fn test_frozen() {
    let mut context = test_context_bounded(1024, 1024);
    let mob_pos = WorldPosition::new(1, 1);
    let mob_chunk = ChunkIndex::from_world_pos(mob_pos);
    let mob = {
        let mut world = &mut context.state.world;
        place_mob(&mut world, mob_pos)
    };

    assert!(context.state.world.entities_in_chunk(&mob_chunk).contains(&mob));

    state::run_action_no_ai(&mut context, Action::TeleportUnchecked(WorldPosition::new(1023, 1023)));

    assert_eq!(
        context.state.world.frozen_in_chunk(&ChunkIndex::new(0, 0)),
        vec![mob]
    );
    assert_eq!(
        context.state.world.spatial.get(mob),
        Some(Place::Unloaded(mob_pos))
    );

    state::run_action_no_ai(&mut context, Action::TeleportUnchecked(WorldPosition::new(0, 0)));

    assert_eq!(
        context.state.world.position(mob),
        Some(mob_pos)
    );
    assert_eq!(
        context.state.world.spatial.get(mob),
        Some(Place::At(mob_pos))
    );
    assert!(context.state.world.entities_in_chunk(&mob_chunk).contains(&mob));
}

#[test]
fn test_load_twice() {
    let mut context = test_context_bounded(1024, 1024);
    let idx = ChunkIndex::new(0, 0);
    context.state.world.load_chunk(&idx).unwrap();
    context.state.world.load_chunk(&idx).unwrap();
}


#[cfg(never)]
#[test]
fn test_load_modify_terrain() {
    let mut world = World::new().with_bounds(Bounds::Bounded(64, 64))
        .with_seed(1)
        .build();
    let change_pos = Point::new(0, 0);

    let cell_mut = world.cell_mut(&change_pos);
    assert!(cell_mut.is_some(), "World terrain wasn't loaded in before mutate");
}

