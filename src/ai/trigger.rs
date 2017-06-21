use std::collections::HashMap;

use calx_ecs::Entity;

use ecs::traits::*;
use logic::entity::EntityQuery;
use world::traits::Query;
use world::World;

use super::{Ai, AiFacts, AiGoal, AiKind};

// TODO: If something strange happens during an unrelated AI goal, the AI should be able to react.
// The obvious example is a neutral entity being attacked by something, in which case they could
// run or turn hostile.
#[derive(Serialize, Deserialize, Hash, Ord, PartialOrd, Eq, PartialEq, Debug, Clone, Copy)]
pub enum AiTrigger {
    AttackedBy(Entity),
    EntityWeak(Entity),
    SawEntity(Entity),
    FriendAttacks(Entity),
    FriendDied,
    TargetLost,
    TargetInRange,
    TargetOutOfRange,
    HealthLow,
}

impl AiKind {
    pub fn check_triggers(&self, entity: Entity, world: &World) -> Option<(AiGoal, Option<Entity>)> {
        let ai = world.ecs().ais.get_or_err(entity);
        let ai_goal = ai.last_goal.borrow();
        let triggers = ai.triggers.borrow();
        let mut res = None;

        for trigger in triggers.iter() {
            if let Some(r) = self.check_trigger(entity, world, *ai_goal, *trigger) {
                res = Some(r);
            }
        }

        res
    }

    fn check_trigger(&self, entity: Entity, world: &World, goal: AiGoal, trigger: AiTrigger) -> Option<(AiGoal, Option<Entity>)> {
        match *self {
            AiKind::Guard => match goal {
                AiGoal::Guard => match trigger {
                    AiTrigger::SawEntity(seen) => {
                        if !world.is_player(seen) {
                            Some((AiGoal::KillTarget, Some(seen)))
                        } else {
                            None
                        }
                    },
                    _ => None,
                },
                _ => None,
            },
            _ => None,
        }
    }
}

// match kind {
//     AiKind::Civilian => match trigger {
//         AttackedBy(attacker) => trigger_coward(entity, Some(attacker), world),
//         FriendDied => trigger_coward(entity, None, world),
//         _ => (),
//     },
//     AiKind::Melee | AiKind::Ranged | AiKind::RangedCloseIn => match trigger {
//         AttackedBy(attacker) => trigger_angry(entity, attacker, world),
//         _ => (),
//     },
//     AiKind::RangedCloseIn => match trigger {
//         TargetInRange => trigger_close_in(entity, world)
//     },
//     _ => match trigger {
//         TargetLost => default(),
//         _ => (),
//     }
// }

// fn trigger_ranged(entity: Entity, world: &mut World) -> (AiGoal, Option<Entity>) {
//     format_mes!(world, entity, "%U <close in>!");
//     (AiGoal::KillTarget, Some(target))
// }

// fn trigger_close_in(entity: Entity, world: &mut World) -> (AiGoal, Option<Entity>) {
//     format_mes!(world, entity, "%U <close in>!");
//     (AiGoal::KillTarget, Some(target))
// }

// fn trigger_angry(entity: Entity, target: Entity, world: &mut World) -> (AiGoal, Option<Entity>) {
//     (AiGoal::KillTarget, Some(target))
// }

// fn trigger_coward(entity: Entity, fear: Option<Entity> world: &mut World) -> (AiGoal, Option<Entity>) {
//     format_mes!(world, entity, "%U <run away>!");
//     (AiGoal::EscapeDanger, fear)
// }
