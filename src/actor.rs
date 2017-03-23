use action::Action;
use chunk::Chunk;
use point::Point;
use world::World;
use slog::Logger;

use log;

pub struct Actor {
    x: i32,
    y: i32,
    logger: Logger,
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
            logger: log::make_logger("actor").unwrap(),
        }
    }

    pub fn run_action(&mut self, action: Action, world: &mut World) {
        match action {
            Action::Move(dir) => self.move_in_direction(dir, world),
            Action::Dood => println!("Dood!"),
        }
    }

    fn move_in_direction(&mut self, dir: Direction, world: &mut World) {
        let (dx, dy) = dir.to_movement_offset();
        let cx = self.x.clone();
        let cy = self.y.clone();
        self.move_to(Point::new(cx + dx, cy + dy), world);
    }

    fn move_to(&mut self, pos: Point, world: &mut World) {
        if world.is_pos_valid(pos) {
            self.x = pos.x;
            self.y = pos.y;
        } else {
            warn!(self.logger, "Actor tried to move to invalid pos {}", pos);
        }
    }

    pub fn get_pos(&self) -> Point {
        Point::new(self.x, self.y)
    }
}
