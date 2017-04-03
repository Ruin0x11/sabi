use std::cell::RefCell;
use std::cmp;
use std::fmt::{self, Display};

use action::Action;
use direction::Direction;
use glyph::Glyph;
use log;
use point::Point;
use namegen;
use world::{World, WorldPosition, Walkability};
use slog::Logger;
use uuid::Uuid;
use fov::FieldOfView;
use stats::Stats;
use stats::archetype;
use stats::properties::Properties;

const FOV_RADIUS: i32 = 10;

lazy_static! {
    static ref ACTOR_LOG: Logger = log::make_logger("actor").unwrap();
}

pub type ActorId = Uuid;

#[derive(Eq, PartialEq)]
pub enum Disposition {
    Friendly,
    Enemy,
}

pub struct Actor {
    // TEMP: The player can name things, names can have pre/suffixes, creatures
    // should be named by their breed, creature variations make their own
    // pre/suffixes, things can have proper names...
    pub name: String,

    x: i32,
    y: i32,

    hit_points: i32,

    uuid: Uuid,

    // TEMP
    pub glyph: Glyph,
    // TEMP
    pub speed: u32,

    pub disposition: Disposition,
    pub logger: Logger,
    pub stats: Stats,
    pub properties: Properties,

    fov: RefCell<FieldOfView>,
}

impl Display for Actor {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} ({:8})...", self.name(), self.get_id())
    }
}

impl Actor {
    // TODO: Should never be used. Use archetypes instead.
    pub fn new(x: i32, y: i32, glyph: Glyph) -> Self {
        let id = Uuid::new_v4();
        Actor {
            // TEMP: Things that can be looked up in a hashmap.
            glyph: glyph,

            // TEMP: Things generated at creation.
            name: namegen::gen(),
            hit_points: 100,
            speed: 100,

            stats: Stats::default(),
            properties: Properties::new(),
            disposition: Disposition::Enemy,

            // Things needing instantiation.
            x: x,
            y: y,
            logger: Actor::get_actor_log(&id),
            uuid: id,
            fov: RefCell::new(FieldOfView::new()),
        }
    }

    pub fn from_archetype(x: i32, y: i32, archetype_name: &str) -> Self {
        let id = Uuid::new_v4();
        let archetype = archetype::load(archetype_name);
        Actor {
            glyph: archetype.glyph,

            name: namegen::gen(),
            hit_points: archetype.stats.max_hp() as i32,
            speed: 100,

            stats: archetype.stats,
            properties: archetype.properties,
            disposition: Disposition::Enemy,

            x: x,
            y: y,
            logger: Actor::get_actor_log(&id),
            uuid: id,
            fov: RefCell::new(FieldOfView::new()),
        }
    }

    fn get_actor_log(id: &ActorId) -> Logger {
        ACTOR_LOG.new(o!("id" => format!("{:.8}...", id.to_string())))
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn move_in_direction(&mut self, dir: Direction, world: &mut World) {
        let pos = Direction::add_offset(self.get_pos(), dir);

        self.move_to(pos, world);
    }

    pub fn move_to(&mut self, pos: Point, world: &mut World) {
        // TEMP: You could displace monsters later.
        if world.pos_valid(&pos) && world.is_walkable(pos, Walkability::MonstersBlocking) {
            world.pre_update_actor_pos(self.get_pos(), pos);
            self.x = pos.x;
            self.y = pos.y;
        } else {
            warn!(self.logger, "Actor tried moving to blocked pos: {}", pos);
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
    pub fn fov<'a>(&self) -> FieldOfView {
        self.fov.borrow().clone()
    }

    pub fn is_player(&self, world: &World) -> bool {
        world.player_id() == self.get_id()
    }

    pub fn is_dead(&self) -> bool {
        self.hit_points <= 0
    }

    pub fn hurt(&mut self, amount: u32) {
        self.hit_points -= amount as i32;
        if self.hit_points <= 0 {
            // TODO: Death.
        }
    }

    pub fn kill(&mut self) {
        let mhp = self.stats.max_hp();
        self.hurt(mhp);
    }
}
