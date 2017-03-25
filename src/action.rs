use actor::Direction;

#[derive(Debug)]
pub enum Action {
    Move(Direction),
    Wait,
    Dood,
}
