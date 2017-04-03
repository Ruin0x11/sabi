use actor::{Actor};
use direction::Direction;
use action::Action;
use world::{World, Walkability};
use pathfinding::Path;
use drawcalls::Draw;

pub struct Simple;

pub trait Ai {
    fn choose_action(&self, action: &Actor, world: &World) -> Action;
}

// TEMP: This should be the behavior of pursuing the player, not the entire AI
impl Ai for Simple {
    fn choose_action(&self, actor: &Actor, world: &World) -> Action {
        let player = world.player();
        if player.is_dead() {
            return Action::Wait;
        }

        let my_pos = actor.get_pos();
        let player_pos = player.get_pos();

        if !actor.can_see(&player_pos) {
            return Action::Wait; // TEMP: or wander
        }

        // Am I right next to the player?
        match Direction::from_neighbors(my_pos, player_pos) {
            Some(dir) => return Action::Move(dir),
            None      => (),
        }

        let mut path = Path::find(my_pos, player_pos, &world, Walkability::MonstersBlocking);

        debug!(actor.logger, "My: {} player: {}, path: {:?}", my_pos, player_pos, path);

        if path.len() == 0 {
            return Action::Wait;
        }

        let next_pos = path.next().unwrap();

        for pt in path {
            world.draw_calls.push(Draw::Point(pt.x, pt.y));
        }

        match Direction::from_neighbors(my_pos, next_pos) {
            Some(dir) => Action::Move(dir),
            None      => panic!("Can't traverse path: {} {}", my_pos, next_pos),
        }
    }
}

// TEMP: should be a behaviour
fn wander() -> Action {
    Action::Move(Direction::choose8())
}
