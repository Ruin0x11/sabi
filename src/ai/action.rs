use calx_ecs::Entity;

use ecs::traits::*;
use point::Direction;
use logic::Action;
use data::Walkability;
use point::Path;
use world::traits::*;
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
    let target_pos = world.position(target).unwrap();

    assert!(world.can_see(entity, target_pos), "Entity can't see target!");

    if my_pos.is_next_to(target_pos) {
        return Action::Move(Direction::from_neighbors(my_pos, target_pos).unwrap());
    }

    let mut path = Path::find(my_pos, target_pos, world, Walkability::MonstersBlocking);

    if path.len() == 0 {
        path = Path::find(my_pos, target_pos, world, Walkability::MonstersWalkable);

        if path.len() == 0 {
            return wander(entity, world);
        }
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
