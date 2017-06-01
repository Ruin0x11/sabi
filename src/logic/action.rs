use calx_ecs::Entity;

use point::{Direction, Point};

#[derive(Clone, Debug)]
pub enum Action {
    Move(Direction),
    MoveOrAttack(Direction),
    Wait,
    SwingAt(Entity),

    TestScript,

    TeleportUnchecked(Point),
}
