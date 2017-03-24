use action::Action;
use chunk::Chunk;
use glyph::Glyph;
use point::Point;
use world::World;
use slog::Logger;
use uuid::Uuid;

use log;

pub type ActorId = Uuid;

pub struct Actor {
    x: i32,
    y: i32,
    pub glyph: Glyph,

    logger: Logger,
    uuid: Uuid,

    pub speed: u32,
}

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

    fn from_movement_offset(offset: Point) -> Option<Direction> {
        let (x, y) = (offset.x, offset.y);
        match (x, y) {
            (0,  -1) => Some(Direction::N),
            (1,  -1) => Some(Direction::NE),
            (1,   0) => Some(Direction::E),
            (1,   1) => Some(Direction::SE),
            (0,   1) => Some(Direction::S),
            (-1,  1) => Some(Direction::SW),
            (-1,  0) => Some(Direction::W),
            (-1, -1) => Some(Direction::NW),
            _        => None,
        }
    }

    pub fn from_neighbors(from: Point, to: Point) -> Option<Direction> {
        Direction::from_movement_offset(from - to)
    }
}

impl Actor {
    pub fn new(x: i32, y: i32, glyph: Glyph) -> Self {
        Actor {
            x: x,
            y: y,
            logger: log::make_logger("actor").unwrap(),
            glyph: glyph,
            uuid: Uuid::new_v4(),
            speed: 100,
        }
    }

    pub fn run_action(&mut self, action: Action, world: &mut World) {
        match action {
            Action::Move(dir) => self.move_in_direction(dir, world),
            Action::Dood => println!("Dood!"),
            Action::Wait => (),
        }
    }

    fn move_in_direction(&mut self, dir: Direction, world: &mut World) {
        let (dx, dy) = dir.to_movement_offset();
        let cx = self.x.clone();
        let cy = self.y.clone();
        self.move_to(Point::new(cx + dx, cy + dy), world);
    }

    fn move_to(&mut self, pos: Point, world: &mut World) {
        if world.is_pos_in_bounds(pos) {
            self.x = pos.x;
            self.y = pos.y;
        } else {
            warn!(self.logger, "Actor tried to move outside of loaded world! {}", pos);
        }
    }

    pub fn advance_time(&mut self, ticks: u32) {
        
    }

    pub fn get_pos(&self) -> Point {
        Point::new(self.x, self.y)
    }

    pub fn get_id(&self) -> Uuid {
        self.uuid
    }

    pub fn is_player(&self, world: &World) -> bool {
        world.player_id() == self.get_id()
    }
}
