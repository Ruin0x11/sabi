use action::Action;
use chunk::Chunk;
use glyph::Glyph;
use point::Point;
use world::World;
use slog::Logger;
use uuid::Uuid;

use log;

pub struct Actor {
    x: i32,
    y: i32,
    pub glyph: Glyph,

    logger: Logger,
    uuid: Uuid,

    pub speed: u32,
    ticks_since_update: u32,
    active: bool,
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
    pub fn new(x: i32, y: i32, glyph: Glyph) -> Self {
        Actor {
            x: x,
            y: y,
            logger: log::make_logger("actor").unwrap(),
            glyph: glyph,
            uuid: Uuid::new_v4(),
            speed: 100,
            active: false,
            ticks_since_update: 0,
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
        if world.is_pos_valid(pos) {
            self.x = pos.x;
            self.y = pos.y;
        } else {
            warn!(self.logger, "Actor tried to move outside of loaded world! {}", pos);
        }
    }

    pub fn pass_time(&mut self, ticks: u32) {
        self.ticks_since_update += ticks;
        if self.ticks_since_update > self.speed {
            self.ticks_since_update %= self.speed;
            self.active = true;
        }
    }

    pub fn check_and_reset_is_active(&mut self) -> bool {
        let result = self.active;
        self.active = false;
        result
    }

    pub fn get_pos(&self) -> Point {
        Point::new(self.x, self.y)
    }

    pub fn get_uuid(&self) -> Uuid {
        self.uuid
    }
}
