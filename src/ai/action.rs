use calx_ecs::Entity;
use goap::*;

use ecs::traits::*;
use point::Direction;
use logic::Action;
use data::Walkability;
use point::Path;
use world::traits::*;
use world::World;

use super::{Ai, AiProp};

#[derive(Serialize, Deserialize, Hash, Ord, PartialOrd, Eq, PartialEq, Debug, Clone)]
pub enum AiAction {
    Wander,
    MoveCloser,
    SwingAt,
    Run,
}

pub(super) fn choose_action(entity: Entity, world: &World) -> Action {
    // TEMP: Just save the whole plan and only update when something interesting
    // happens
    let ai = world.ecs().ais.get_or_err(entity);

    match *ai.next_action.borrow() {
        Some(ref action) => {
            match *action {
                AiAction::Wander => ai_wander(entity, world),
                AiAction::MoveCloser => ai_move_closer(entity, world),
                AiAction::SwingAt => ai_swing_at(entity, world),
                AiAction::Run => ai_run_away(entity, world),
            }
        },
        None => {
            warn_of_unreachable_states(entity, world, &ai);
            Action::Wait
        },
    }
}


fn ai_wander(_entity: Entity, _world: &World) -> Action {
    Action::Move(Direction::choose8())
}

fn ai_move_closer(entity: Entity, world: &World) -> Action {
    match direction_towards_target(entity, world) {
        Some(dir) => Action::Move(dir),
        None => Action::Wait,
    }
}

fn ai_swing_at(entity: Entity, world: &World) -> Action {
    let ais = &world.ecs().ais;
    let ai = ais.get_or_err(entity);

    Action::SwingAt(ai.target.borrow().unwrap())
}

fn ai_run_away(entity: Entity, world: &World) -> Action {
    match direction_towards_target(entity, world) {
        Some(dir) => Action::Move(dir.reverse()),
        None => Action::Wait,
    }
}


fn direction_towards(entity: Entity, target: Entity, world: &World) -> Option<Direction> {
    let my_pos = world.position(entity).unwrap();
    let target_pos = world.position(target).unwrap();

    // assert!(entity.can_see_other(target, world), "Entity can't see target!");

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

    Some(Direction::from_neighbors(my_pos, next_pos).unwrap())
}

fn direction_towards_target(entity: Entity, world: &World) -> Option<Direction> {
    let ais = &world.ecs().ais;
    let ai = ais.get_or_err(entity);

    let target = ai.target.borrow().unwrap();
    assert!(world.is_alive(target), "Target is already dead!");
    direction_towards(entity, target, world)
}


fn warn_of_unreachable_states(entity: Entity, world: &World, ai: &Ai) {
    warn_ecs!(world, entity, "I can't figure out what to do! \nfrom: {:?}\nto:{:?}",
              ai.memory.borrow(), ai.goal.borrow());
    if let Err(failed_state) = ai.get_plan() {
        let mut needed: Vec<AiProp> = ai.goal.borrow().facts.iter().filter(|&(cond, val)| {
            failed_state.facts.get(cond).map_or(false, |f| f != val)
        }).map(|(cond, _)| cond.clone()).collect();

        for action in ai.planner.get_actions().into_iter() {
            let effects = ai.planner.actions(action);
            let satisfied: Vec<AiProp> = effects.postconditions.iter().filter(|&(cond, val)| {
                failed_state.facts.get(cond).map_or(true, |f| f == val)
            }).map(|(cond, _)| cond.clone()).collect();

            for s in satisfied {
                needed.retain(|u| *u != s);
            }
        }

        warn_ecs!(world, entity, "No actions could be found to make these properties true:");
        warn_ecs!(world, entity, "{:?}", needed);
    }
}
