use std::cell::RefCell;

use action::Action;
use glyph::Glyph;
use point::Point;
use world::{World, WorldPosition, Walkability};
use slog::Logger;
use uuid::Uuid;
use fov::FieldOfView;

use log;

const FOV_RADIUS: i32 = 5;

lazy_static! {
    static ref ACTOR_LOG: Logger = log::make_logger("actor").unwrap();
}

pub type ActorId = Uuid;

pub struct Actor {
    x: i32,
    y: i32,
    pub glyph: Glyph,

    pub logger: Logger,
    uuid: Uuid,

    pub speed: u32,

    fov: RefCell<FieldOfView>,
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
            Direction::NW => (-1, -1),
            Direction::W  => (-1,  0),
            Direction::SW => (-1,  1),
            Direction::S  => (0,   1),
            Direction::SE => (1,   1),
            Direction::E  => (1,   0),
            Direction::NE => (1,  -1),
        }
    }

    fn from_movement_offset(offset: Point) -> Option<Direction> {
        let (x, y) = (offset.x, offset.y);
        match (x, y) {
            (0,  -1) => Some(Direction::N),
            (-1, -1) => Some(Direction::NW),
            (-1,  0) => Some(Direction::W),
            (-1,  1) => Some(Direction::SW),
            (0,   1) => Some(Direction::S),
            (1,   1) => Some(Direction::SE),
            (1,   0) => Some(Direction::E),
            (1,  -1) => Some(Direction::NE),
            _        => None,
        }
    }

    pub fn from_neighbors(from: Point, to: Point) -> Option<Direction> {
        Direction::from_movement_offset(to - from)
    }
}

impl Actor {
    pub fn new(x: i32, y: i32, glyph: Glyph) -> Self {
        let id = Uuid::new_v4();
        Actor {
            x: x,
            y: y,
            logger: ACTOR_LOG.new(o!("id" => format!("{:.8}...", id.to_string()))),
            glyph: glyph,
            uuid: id,
            speed: 100,
            fov: RefCell::new(FieldOfView::new()),
        }
    }

    pub fn run_action(&mut self, action: Action, world: &mut World) {
        match action {
            Action::Move(dir) => self.move_in_direction(dir, world),
            Action::Dood => world.message("Dood!".to_string()),
            Action::Wait => (),
        }
    }

    fn move_in_direction(&mut self, dir: Direction, world: &mut World) {
        let (dx, dy) = dir.to_movement_offset();
        let cx = self.x.clone() + dx;
        let cy = self.y.clone() + dy;
        let pos = Point::new(cx, cy);

        if world.is_walkable(pos, Walkability::MonstersBlocking) {
            self.move_to(pos, world);
        }
    }

    fn move_to(&mut self, pos: Point, world: &mut World) {
        if world.pos_valid(&pos) {
            world.pre_update_actor_pos(self.get_pos(), pos);
            self.x = pos.x;
            self.y = pos.y;
        } else {
            warn!(self.logger, "Actor tried to move outside of loaded world! {}", pos);
        }
    }

    pub fn get_pos(&self) -> Point {
        Point::new(self.x, self.y)
    }

    pub fn get_id(&self) -> Uuid {
        self.uuid
    }

    pub fn update_fov(&self, world: &World) {
        self.fov.borrow_mut().clear();

        let in_bounds = |pos: &WorldPosition| world.pos_valid(pos);
        let blocked = |pos: &WorldPosition| !world.cell(pos).unwrap().tile.can_pass_through();

        self.fov.borrow_mut().update(&self.get_pos(), FOV_RADIUS, in_bounds, blocked);
    }

    pub fn can_see(&self, pos: &WorldPosition) -> bool {
        self.fov.borrow().is_visible(pos)
    }

    // FIXME: to satisfy the borrow checker
    pub fn fov(&self) -> FieldOfView {
        self.fov.borrow().clone()
    }

    pub fn is_player(&self, world: &World) -> bool {
        world.player_id() == self.get_id()
    }
}
