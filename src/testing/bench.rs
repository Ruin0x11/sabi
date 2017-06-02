use test::Bencher;

use calx_ecs::Entity;

use logic::Action;
use testing::*;
use state;
use world::traits::*;
use world::WorldPosition;

fn many_entities() -> GameContext {
    let mut context = test_context_bounded(128, 128);
    for i in 1..32 {
        for j in 1..32 {
            place_mob(&mut context.state.world, WorldPosition::new(i, j));
        }
    }
    context
}

#[bench]
fn bench_many_entities(b: &mut Bencher) {
    let mut context = many_entities();

    b.iter(|| {
        state::run_action(&mut context, Action::Wait);
    });
}

use renderer::RenderContext;

#[bench]
fn bench_renderer_update(b: &mut Bencher) {
    let context = many_entities();
    let mut renderer = RenderContext::new();

    b.iter(|| {
        renderer.update(&context);
        renderer.render();
    });
}

#[bench]
fn bench_renderer_idle(b: &mut Bencher) {
    let context = many_entities();
    let mut renderer = RenderContext::new();

    renderer.update(&context);
    b.iter(|| {
        renderer.render();
    });
}

#[bench]
fn bench_fov(b: &mut Bencher) {
    let mut context = many_entities();
    let world = &mut context.state.world;

    b.iter(|| {
        let entities: Vec<Entity> = world.entities().cloned().collect();
        for e in entities.iter() {
            world.do_fov(*e);
        }
    });
}
