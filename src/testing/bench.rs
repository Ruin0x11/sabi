use test::Bencher;

use calx_ecs::Entity;

use logic::Action;
use testing::*;
use state;
use world::traits::*;
use world::WorldPosition;

fn many_entities() -> GameContext {
    let mut context = test_context_bounded(128, 128);
    for i in 0..32 {
        for j in 0..32 {
            place_mob(&mut context.state.world, WorldPosition::new(i, j));
        }
    }
    context
}

#[bench]
fn bench_ai(b: &mut Bencher) {
    let mut context = test_context_bounded(128, 128);
    place_mob(&mut context.state.world, WorldPosition::new(64, 64));

    b.iter(|| { state::run_action(&mut context, Action::Wait); });
}

#[bench]
fn bench_no_ai(b: &mut Bencher) {
    let mut context = test_context_bounded(128, 128);
    place_mob(&mut context.state.world, WorldPosition::new(64, 64));

    b.iter(|| { state::run_action_no_ai(&mut context, Action::Wait); });
}

#[bench]
#[ignore]
fn bench_many_entities(b: &mut Bencher) {
    let mut context = many_entities();

    b.iter(|| { state::run_action(&mut context, Action::Wait); });
}

#[bench]
#[ignore]
fn bench_many_entities_no_ai(b: &mut Bencher) {
    let mut context = many_entities();

    b.iter(|| { state::run_action_no_ai(&mut context, Action::Wait); });
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

use renderer::RenderContext;

#[bench]
#[ignore]
fn bench_renderer_update(b: &mut Bencher) {
    let context = many_entities();
    let mut renderer = RenderContext::new();

    b.iter(|| {
               renderer.update(&context.state);
               renderer.render();
           });
}

#[bench]
#[ignore]
fn bench_renderer_idle(b: &mut Bencher) {
    let context = many_entities();
    let mut renderer = RenderContext::new();

    renderer.update(&context.state);
    b.iter(|| { renderer.render(); });
}
