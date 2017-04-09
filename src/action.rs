use actor::ActorId;
use direction::Direction;

#[derive(Clone, Debug)]
pub enum Action {
    Move(Direction),
    Wait,
    Dood,
    Explod,
    Hurt(u32),
    SwingAt(ActorId),
}
