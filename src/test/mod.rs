use calx_ecs::Entity;

use ecs::Ecs;
use ecs::prefab::Prefab;
use point::Point;
use state;
use world::traits::Mutate;
use world::{EcsWorld, WorldPosition};

use ::GameContext;

pub fn get_ecs() -> Ecs {
    Ecs::new()
}

pub fn get_world_bounded(w: i32, h: i32) -> EcsWorld {
    let mut world = EcsWorld::new_blank(w, h);
    let e = world.create(::ecs::prefab::mob("Player", 100000, ::glyph::Glyph::Player), Point::new(0,0));
    world.set_player(Some(e));
    world
}

pub fn test_context_bounded(w: i32, h: i32) -> GameContext {
    let mut context = GameContext::new();
    context.state.world = get_world_bounded(w, h);
    state::process(&mut context);
    context
}

pub fn place_mob(world: &mut EcsWorld, pos: WorldPosition) -> Entity {
    world.create(Prefab::new(), pos)
}
