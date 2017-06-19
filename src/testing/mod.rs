mod bench;

use calx_ecs::Entity;

use ai::*;
use ecs;
use state;
use point::Point;
use world::traits::Mutate;
use world::{Bounds, World, WorldPosition};

use GameContext;

pub fn blank_world(w: i32, h: i32) -> World {
    World::new()
        .with_bounds(Bounds::Bounded(w, h))
        .build()
        .unwrap()
}

pub fn get_world_bounded(w: i32, h: i32) -> World {
    let mut world = blank_world(w, h);
    let e = world.create(ecs::prefab::mob("player", 1000000, "player"), Point::new(0, 0));
    world.set_player(Some(e));
    world
}

pub fn test_context() -> GameContext {
    test_context_bounded(32, 32)
}

pub fn test_context_bounded(w: i32, h: i32) -> GameContext {
    let mut context = GameContext::new();
    context.state.world = get_world_bounded(w, h);
    state::init_game_context(&mut context);
    context
}

pub fn place_mob(world: &mut World, pos: WorldPosition) -> Entity {
    world.create(ecs::prefab::mob("mob", 100, "putit").c(Ai::new(AiKind::Wander)), pos)
}
