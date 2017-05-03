mod bench;

use calx_ecs::Entity;

use ecs::Ecs;
use ecs::prefab;
use graphics::Glyph;
use state;
use point::Point;
use chunk::generator::ChunkType;
use world::traits::Mutate;
use world::{Bounds, EcsWorld, WorldPosition};

use ::GameContext;

pub fn get_ecs() -> Ecs {
    Ecs::new()
}

pub fn blank_world(w: i32, h: i32) -> EcsWorld {
    let world = EcsWorld::new(Bounds::Bounded(w, h), ChunkType::Blank, 1, 0);
    world
}

pub fn get_world_bounded(w: i32, h: i32) -> EcsWorld {
    let mut world = blank_world(w, h);
    let e = world.create(prefab::mob("Player", 100000, Glyph::Player), Point::new(0,0));
    world.set_player(Some(e));
    world
}

pub fn test_context_bounded(w: i32, h: i32) -> GameContext {
    let mut context = GameContext::new();
    context.state.world = get_world_bounded(w, h);
    state::init_headless(&mut context);
    context
}

pub fn place_mob(world: &mut EcsWorld, pos: WorldPosition) -> Entity {
    world.create(prefab::mob("Mob", 100, Glyph::Putit), pos)
}
