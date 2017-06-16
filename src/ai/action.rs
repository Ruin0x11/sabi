use calx_ecs::Entity;

use ecs::traits::*;
use point::Direction;
use logic::Action;
use logic::entity::EntityQuery;
use data::Walkability;
use point::Path;
use world::traits::*;
use world::World;

// TODO: Allow variable arguments, since we have no need to follow a consistent
// API?

pub fn ai_wander(_entity: Entity, _world: &World) -> Action {
    Action::Move(Direction::choose8())
}

pub fn ai_swing_at(entity: Entity, world: &World) -> Action {
    let ais = &world.ecs().ais;
    let ai = ais.get_or_err(entity);

    Action::SwingAt(ai.target.borrow().unwrap())
}

fn direction_towards(entity: Entity, target: Entity, world: &World) -> Option<Direction> {
    let my_pos = world.position(entity).unwrap();
    let target_pos = world.position(target).unwrap();

    assert!(entity.has_los(target_pos, world),
            "Entity can't see target!");

    if my_pos.is_next_to(target_pos) {
        return Direction::from_neighbors(my_pos, target_pos);
    }

    let mut path = Path::find(my_pos, target_pos, world, Walkability::MonstersBlocking);

    if path.len() == 0 {
        path = Path::find(my_pos, target_pos, world, Walkability::MonstersWalkable);

        if path.len() == 0 {
            return None;
        }
    }

    let next_pos = path.next().unwrap();

    // for pt in path {
    // world.draw_calls.push(Draw::Point(pt.x, pt.y));
    // }

    Some(Direction::from_neighbors(my_pos, next_pos).unwrap())
}

fn direction_towards_target(entity: Entity, world: &World) -> Option<Direction> {
    let ais = &world.ecs().ais;
    let ai = ais.get_or_err(entity);

    let target = ai.target.borrow().unwrap();
    assert!(world.is_alive(target), "Target is already dead!");
    direction_towards(entity, target, world)
}

pub fn ai_move_closer(entity: Entity, world: &World) -> Action {
    match direction_towards_target(entity, world) {
        Some(dir) => Action::Move(dir),
        None => Action::Wait,
    }
}

pub fn ai_run_away(entity: Entity, world: &World) -> Action {
    match direction_towards_target(entity, world) {
        Some(dir) => Action::Move(dir.reverse()),
        None => Action::Wait,
    }
}
