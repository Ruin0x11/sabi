use calx_ecs::Entity;

use direction::Direction;
use action::Action;
use data::Walkability;
use pathfinding::Path;
use drawcalls::Draw;
use ecs::traits::*;
use world::EcsWorld;

// TODO: Allow variable arguments, since we have no need to follow a consistent
// API?

pub fn wander(_entity: Entity, _world: &EcsWorld) -> Action {
    Action::Move(Direction::choose8())
}

pub fn swing_at(entity: Entity, world: &EcsWorld) -> Action {
    let ref ais = world.ecs().ais;
    let ai = ais.get_or_err(entity);

    Action::SwingAt(ai.target.borrow().unwrap())
}

pub fn move_closer(entity: Entity, world: &EcsWorld) -> Action {
    let ref ais = world.ecs().ais;
    let ai = ais.get_or_err(entity);

    let target = ai.target.borrow().unwrap();
    assert!(world.is_alive(target), "Target is already dead!");

    let my_pos = world.position(entity).unwrap();
    let target_pos = world.position(entity).unwrap();

    // assert!(entity.can_see(&target_pos), "Entity can't see target!");

    // Am I right next to the target?
    match Direction::from_neighbors(my_pos, target_pos) {
        Some(dir) => return Action::Move(dir),
        None      => (),
    }

    let mut path = Path::find(my_pos, target_pos, world, Walkability::MonstersBlocking);

    // debug!(entity.logger, "My: {} target: {}, path: {:?}", my_pos, target_pos, path);

    if path.len() == 0 {
        // TODO: Lost sight of target.
        return Action::Wait;
    }

    let next_pos = path.next().unwrap();

    // for pt in path {
        // world.draw_calls.push(Draw::Point(pt.x, pt.y));
    // }

    match Direction::from_neighbors(my_pos, next_pos) {
        Some(dir) => Action::Move(dir),
        None      => panic!("Can't traverse path: {} {}", my_pos, next_pos),
    }
}

pub fn run_away(_entity: Entity, _world: &EcsWorld) -> Action {
    Action::Move(Direction::choose8())
}
