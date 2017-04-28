use calx_ecs::Entity;

use point::{Direction, Point};

#[derive(Clone, Debug)]
pub enum Action {
    Move(Direction),
    MoveOrAttack(Direction),
    Wait,
    Dood,
    Explod,
    Hurt(u32),
    SwingAt(Entity),

    Teleport(Point),
    TeleportUnchecked(Point),
}
