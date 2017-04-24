use calx_ecs::Entity;

use direction::Direction;
use action::Action;
use world::{World, Walkability};
use pathfinding::Path;
use drawcalls::Draw;

pub fn wander(_entity: &Entity, _world: &World) -> Action {
    Action::Move(Direction::choose8())
}

pub fn swing_at(entity: &Entity, _world: &World) -> Action {
    // Action::SwingAt(entity.ai.target.borrow().unwrap())
    Action::Wait
}

pub fn move_closer(entity: &Entity, world: &World) -> Action {
    Action::Wait
    // let target = world.entity(&entity.ai.target.borrow().unwrap());
    // assert!(!target.is_dead(), "Target is already dead!");

    // let my_pos = entity.get_pos();
    // let target_pos = target.get_pos();

    // assert!(entity.can_see(&target_pos), "Entity can't see target!");

    // // Am I right next to the target?
    // match Direction::from_neighbors(my_pos, target_pos) {
    //     Some(dir) => return Action::Move(dir),
    //     None      => (),
    // }

    // let mut path = Path::find(my_pos, target_pos, &world, Walkability::MonstersBlocking);

    // debug!(entity.logger, "My: {} target: {}, path: {:?}", my_pos, target_pos, path);

    // if path.len() == 0 {
    //     // TODO: Lost sight of target.
    //     return Action::Wait;
    // }

    // let next_pos = path.next().unwrap();

    // for pt in path {
    //     world.draw_calls.push(Draw::Point(pt.x, pt.y));
    // }

    // match Direction::from_neighbors(my_pos, next_pos) {
    //     Some(dir) => Action::Move(dir),
    //     None      => panic!("Can't traverse path: {} {}", my_pos, next_pos),
    // }
}

pub fn run_away(_entity: &Entity, _world: &World) -> Action {
    Action::Move(Direction::choose8())
}
