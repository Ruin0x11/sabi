use calx_ecs::Entity;

use direction::Direction;

#[derive(Clone, Debug)]
pub enum Action {
    Move(Direction),
    MoveOrAttack(Direction),
    Wait,
    Dood,
    Explod,
    Hurt(u32),
    SwingAt(Entity),
}
