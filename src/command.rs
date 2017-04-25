use direction::Direction;
use keys::{Key, KeyCode};

pub type CommandResult = Result<(), ()>;

pub enum Command {
    Move(Direction),
    Wait,
    Quit,
}

impl Command {
    pub fn from_key(key: Key) -> Command {
        match key {
            Key { code: KeyCode::Esc,     .. } => Command::Quit,
            Key { code: KeyCode::Left,    .. } |
            Key { code: KeyCode::H,       .. } |
            Key { code: KeyCode::NumPad4, .. } => Command::Move(Direction::W),
            Key { code: KeyCode::Right,   .. } |
            Key { code: KeyCode::L,       .. } |
            Key { code: KeyCode::NumPad6, .. } => Command::Move(Direction::E),
            Key { code: KeyCode::Up,      .. } |
            Key { code: KeyCode::K,       .. } |
            Key { code: KeyCode::NumPad8, .. } => Command::Move(Direction::N),
            Key { code: KeyCode::Down,    .. } |
            Key { code: KeyCode::J,       .. } |
            Key { code: KeyCode::NumPad2, .. } => Command::Move(Direction::S),
            Key { code: KeyCode::NumPad1, .. } => Command::Move(Direction::SW),
            Key { code: KeyCode::NumPad3, .. } => Command::Move(Direction::SE),
            Key { code: KeyCode::NumPad7, .. } => Command::Move(Direction::NW),
            Key { code: KeyCode::NumPad9, .. } => Command::Move(Direction::NE),
            _                                  => Command::Wait
        }
    }
}
