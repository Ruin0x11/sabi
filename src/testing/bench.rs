use test::Bencher;

use logic::Action;
use testing::*;
use state;
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
    let mut context = many_entities();
    let mut renderer = RenderContext::new();

    b.iter(|| {
        renderer.update(&context);
        renderer.render();
    });
}

#[bench]
fn bench_renderer_idle(b: &mut Bencher) {
    let mut context = many_entities();
    let mut renderer = RenderContext::new();

    renderer.update(&context);
    b.iter(|| {
        renderer.render();
    });
}
