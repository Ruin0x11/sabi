use test::Bencher;

use action::Action;
use testing::*;
use state;
use world::WorldPosition;

#[bench]
fn bench_many_entities(b: &mut Bencher) {
    let mut context = test_context_bounded(128, 128);
    for i in 1..32 {
        for j in 1..32 {
            place_mob(&mut context.state.world, WorldPosition::new(i, j));
        }
    }

    b.iter(|| {
        state::run_action(&mut context, Action::Wait);
    });
}
