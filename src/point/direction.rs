use std::fmt;
use std::ops::Add;
use std::slice::Iter;

use rand::{self, Rng};

use point::Point;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

pub static DIRECTIONS: [Direction; 8] = [
    Direction::N,
    Direction::NE,
    Direction::E,
    Direction::SE,
    Direction::S,
    Direction::SW,
    Direction::W,
    Direction::NW,
];

impl fmt::Display for Direction {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let s = match *self {
            Direction::N => "north",
            Direction::NE => "northeast",
            Direction::E => "east",
            Direction::SE => "southeast",
            Direction::S => "south",
            Direction::SW => "southwest",
            Direction::W => "west",
            Direction::NW => "northwest",
        };
        write!(f, "{}", s)
    }
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

    fn ordinal(&self) -> usize {
        let mut index = 0;
        while index < 7 {
            if DIRECTIONS[index] == *self {
                break;
            }
            index += 1;
        }

        index
    }

    pub fn neighbor(&self, steps: i8) -> Direction {
        let mut ord = self.ordinal() as i8;
        ord += steps;
        ord %= 8;
        if ord < 0 {
            ord = 8 + ord;
        }

        DIRECTIONS[ord as usize]
    }

    pub fn reverse(&self) -> Direction {
        self.neighbor(4)
    }

    pub fn is_straight(&self) -> bool {
        *self == Direction::N || *self == Direction::E || *self == Direction::S ||
            *self == Direction::W
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reverse() {
        assert_eq!(Direction::N.reverse(), Direction::S);
    }

    #[test]
    fn test_neighbor() {
        assert_eq!(Direction::N.neighbor(-1), Direction::NW);
        assert_eq!(Direction::N.neighbor(1), Direction::NE);
        assert_eq!(Direction::N.neighbor(-2), Direction::W);
        assert_eq!(Direction::N.neighbor(2), Direction::E);
    }
}
