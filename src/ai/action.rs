use calx_ecs::Entity;
use goap::*;

use ecs::traits::*;
use point::Direction;
use logic::Action;
use logic::entity::EntityQuery;
use data::Walkability;
use point::{Path, Point};
use rand::{self, Rng};
use world::traits::*;
use world::World;

use ai;
use super::{Ai, AiProp, AiGoal, Target};

macro_rules! generate_ai_actions {
    ( $( $action:ident, $func:ident );+ $(;)*) => {
        macro_attr! {
        #[derive(Serialize, Deserialize, Hash, Ord, PartialOrd, Eq, PartialEq, Debug, Clone, EnumFromStr!)]
            pub enum AiAction {
                $(
                    $action,
                )*
            }
        }

        pub(super) fn choose_action(entity: Entity, world: &World) -> Action {
            // TEMP: Just save the whole plan and only update when something interesting
            // happens
            let ai = world.ecs().ais.get_or_err(entity);

            let result = match *ai.data.next_action.borrow() {
                Some(ref action) => {
                    match *action {
                        $(
                            AiAction::$action => $func(entity, world),
                        )*
                    }
                },
                None => {
                    warn_of_unreachable_states(entity, world, &ai);
                    Action::Wait
                },
            };

            if ai.data.target_was_switched.get() {
                ai.data.target_was_switched.set(false);
                ai::update_next_action(entity, world);
            }

            result
        }

    }
}

generate_ai_actions! {
    Wander, ai_wander;
    MoveCloser, ai_move_closer;
    PickupItem, ai_pickup_item;
    GetThrowable, ai_get_throwable;
    SwingAt, ai_swing_at;
    ShootAt, ai_shoot_at;
    RunAway, ai_run_away;
    ReturnToPosition, ai_return_to_position;
    Wait, ai_wait;
}


fn ai_get_throwable(entity: Entity, world: &World) -> Action {
    let ai = &world.ecs().ais.get_or_err(entity).data;

    let throwable =
        world.seen_entities(entity)
             .into_iter()
             .find(|i| world.is_item(*i) && i.basename(world) == "watermelon");

    if let Some(t) = throwable {
        ai::add_target(Target {
                           entity: Some(t),
                           priority: 90,
                           goal: AiGoal::GetItem,
                       },
                       entity,
                       world);
    }

    Action::Wait
}


fn ai_wait(_entity: Entity, _world: &World) -> Action {
    Action::Wait
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

fn ai_return_to_position(entity: Entity, world: &World) -> Action {
    let ai = &world.ecs().ais.get_or_err(entity).data;

    if let Some(pos) = *ai.important_pos.borrow() {
        match direction_towards(entity, pos, world) {
            Some(dir) => Action::Move(dir),
            None => Action::Wait,
        }
    } else {
        Action::Wait
    }
}

fn ai_pickup_item(entity: Entity, world: &World) -> Action {
    let ai = &world.ecs().ais.get_or_err(entity).data;
    let target = ai.targets.borrow().peek().unwrap().entity.unwrap();
    let items = world.entities_below(entity);
    assert!(items.contains(&target));
    assert!(world.is_item(target));
    Action::Pickup(target)
}

fn ai_swing_at(entity: Entity, world: &World) -> Action {
    let ai = &world.ecs().ais.get_or_err(entity).data;

    Action::SwingAt(ai.targets.borrow().peek().unwrap().entity.unwrap())
}

fn ai_shoot_at(entity: Entity, world: &World) -> Action {
    // TODO: box rng in RefCell
    if rand::thread_rng().gen() {
        return ai_wander(entity, world);
    }

    let ai = &world.ecs().ais.get_or_err(entity).data;
    Action::ShootAt(ai.targets.borrow().peek().unwrap().entity.unwrap())
}

fn ai_run_away(entity: Entity, world: &World) -> Action {
    match direction_towards_target(entity, world) {
        Some(dir) => Action::Move(dir.reverse()),
        None => Action::Wait,
    }
}

fn direction_towards(entity: Entity, target_pos: Point, world: &World) -> Option<Direction> {
    let my_pos = world.position(entity).unwrap();

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
    let ai = &ais.get_or_err(entity).data;

    let target = ai.targets.borrow().peek().unwrap().entity.unwrap();
    let target_pos = world.position(target).unwrap();
    direction_towards(entity, target_pos, world)
}


fn warn_of_unreachable_states(entity: Entity, world: &World, ai: &Ai) {
    warn_ecs!(world, entity, "AI stuck!");
    match ai.data.get_plan() {
        Ok(plan) => {
            warn_ecs!(world, entity, "plan: {:?}", plan);
        },
        Err(failed_state) => {
            let mut needed: Vec<AiProp> =
                ai.data
                  .goal
                  .borrow()
                  .facts
                  .iter()
                  .filter(|&(cond, val)| failed_state.facts.get(cond).map_or(false, |f| f != val))
                  .map(|(cond, _)| cond.clone())
                  .collect();

            ai::instance::with(|planner| for action in planner.get_actions().into_iter() {
                let effects = planner.actions(action);
                let satisfied: Vec<AiProp> = effects.postconditions
                                                    .iter()
                                                    .filter(|&(cond, val)| {
                    failed_state.facts.get(cond).map_or(true, |f| f == val)
                })
                                                    .map(|(cond, _)| cond.clone())
                                                    .collect();

                for s in satisfied {
                    needed.retain(|u| *u != s);
                }
            });

            warn_ecs!(world, entity, "No actions could be found to make these properties true:");
            warn_ecs!(world, entity, "{:?}", needed);
        },
    }
}
