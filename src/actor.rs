use action::Action;
use point::Point;

pub struct Actor {
    x: i32,
    y: i32,
}

#[derive(Clone, Copy)]
pub enum Direction {
    N,
    NE,
    E,
    SE,
    S,
    SW,
    W,
    NW,
}

impl Direction {
    fn to_movement_offset(&self) -> (i32, i32) {
        match *self {
            Direction::N  => (0,  -1),
            Direction::NE => (1,  -1),
            Direction::E  => (1,   0),
            Direction::SE => (1,   1),
            Direction::S  => (0,   1),
            Direction::SW => (-1,  1),
            Direction::W  => (-1,  0),
            Direction::NW => (-1, -1),
        }
    }
}

impl Actor {
    pub fn new(x: i32, y: i32) -> Self {
        Actor {
            x: x,
            y: y,
        }
    }

    pub fn run_action(&mut self, action: Action) {
        match action {
            Action::Move(dir) => self.move_in_direction(dir),
            Action::Dood => println!("Dood!"),
        }
    }

    fn move_in_direction(&mut self, dir: Direction) {
        let (dx, dy) = dir.to_movement_offset();
        let cx = self.x.clone();
        let cy = self.y.clone();
        self.move_to(cx + dx, cy + dy);
    }

    fn move_to(&mut self, nx: i32, ny: i32) {
    // TODO: needs a map/world to check bounds, at minimum
        if true { 
            self.x = nx;
            self.y = ny;
        }
    }

    pub fn get_pos(&self) -> Point {
        Point::new(self.x, self.y)
    }
}
