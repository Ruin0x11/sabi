use action::Action;
use world::World;

struct Simple;

trait Ai {
    fn choose_action(&self, world: &World) -> Action;
}

impl Ai for Simple {
    fn choose_action(&self, world: &World) -> Action {
        Action::Wait
    }
}
