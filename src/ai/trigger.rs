use std::collections::HashMap;

use calx_ecs::Entity;

use ecs::traits::*;
use logic::entity::EntityQuery;
use world::traits::Query;
use world::World;

use super::{Ai, AiFacts, AiGoal};

// TODO: If something strange happens during an unrelated AI goal, the AI should be able to react.
// The obvious example is a neutral entity being attacked by something, in which case they could
// run or turn hostile.
#[derive(Serialize, Deserialize, Hash, Ord, PartialOrd, Eq, PartialEq, Debug, Clone)]
pub enum AiTrigger {
    AttackedBy(Entity),
    EntityWeak(Entity),
    NewTarget(Entity),
    FriendAttacks(Entity),
    FriendDied,
    TargetLost,
    TargetInRange,
    TargetOutOfRange,
    HealthLow,
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
