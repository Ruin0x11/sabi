use std::ops::Add;
use std::slice::Iter;

use rand::{self, Rng};

use point::Point;

pub static DIRECTIONS: [Direction; 8] = [Direction::N,
                                         Direction::NE,
                                         Direction::E,
                                         Direction::SE,
                                         Direction::S,
                                         Direction::SW,
                                         Direction::W,
                                         Direction::NW];


#[derive(Debug, Clone, Copy)]
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
            Direction::N => (0, -1),
            Direction::NW => (-1, -1),
            Direction::W => (-1, 0),
            Direction::SW => (-1, 1),
            Direction::S => (0, 1),
            Direction::SE => (1, 1),
            Direction::E => (1, 0),
            Direction::NE => (1, -1),
        }
    }

    fn from_movement_offset<P: Into<(i32, i32)>>(offset: P) -> Option<Direction> {
        let (x, y) = offset.into();
        match (x, y) {
            (0, -1) => Some(Direction::N),
            (-1, -1) => Some(Direction::NW),
            (-1, 0) => Some(Direction::W),
            (-1, 1) => Some(Direction::SW),
            (0, 1) => Some(Direction::S),
            (1, 1) => Some(Direction::SE),
            (1, 0) => Some(Direction::E),
            (1, -1) => Some(Direction::NE),
            _ => None,
        }
    }

    pub fn reverse(&self) -> Direction {
        let (i, j) = self.to_movement_offset();
        Direction::from_movement_offset((-i, -j)).unwrap()
    }

    pub fn choose8() -> Direction {
        *rand::thread_rng().choose(&DIRECTIONS).unwrap()
    }

    pub fn iter8() -> Iter<'static, Direction> {
        DIRECTIONS.into_iter()
    }

    pub fn from_neighbors(from: Point, to: Point) -> Option<Direction> {
        Direction::from_movement_offset(to - from)
    }
}

impl Add<Direction> for Point {
    type Output = Point;

    fn add(self, dir: Direction) -> Point {
        let (dx, dy) = dir.to_movement_offset();
        let cx = self.x + dx;
        let cy = self.y + dy;
        Point::new(cx, cy)
    }
}
