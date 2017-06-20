use calx_ecs::Entity;
use rand::{self, Rng};

use ai::*;
use logic::entity::EntityQuery;
use ecs::traits::ComponentQuery;
use world::traits::Query;
use world::World;

#[derive(Eq, PartialEq, Clone, Debug, Serialize, Deserialize)]
pub enum AiKind {
    Wait,
    SeekTarget,
    Follow,
    Wander,
    Scavenge,
}

#[derive(Eq, PartialEq, Clone, Debug, Serialize, Deserialize)]
pub enum AiGoal {
    FindTarget,
    KillTarget,
    Follow,
    Wander,
    EscapeDanger,

    FindItem,
    GetItem,

    DoNothing,
}

impl AiGoal {
    pub fn get_end_state(&self) -> AiFacts {
        let mut goal = AiFacts::new();
        for (prop, val) in self.get_props() {
            goal.insert(prop, val);
        }
        goal
    }

    fn get_props(&self) -> Vec<(AiProp, bool)> {
        // TODO: instead make the "health low" things triggers for entering the new goal of "run
        // away and heal"
        match *self {
            AiGoal::FindTarget => vec![(AiProp::TargetVisible, true), (AiProp::HealthLow, false)],
            AiGoal::KillTarget => vec![(AiProp::TargetDead, true), (AiProp::HealthLow, false)],
            AiGoal::Follow => vec![(AiProp::NextToTarget, true)],
            AiGoal::Wander => vec![(AiProp::Moving, true)],

            AiGoal::EscapeDanger => vec![(AiProp::Exists, false)],

            AiGoal::FindItem => vec![(AiProp::FoundItem, true)],
            AiGoal::GetItem => vec![(AiProp::TargetInInventory, true)],

            AiGoal::DoNothing => vec![(AiProp::Exists, false)],
        }
    }

    pub fn requires_target(&self) -> bool {
        match *self {
            AiGoal::GetItem | AiGoal::FindTarget | AiGoal::KillTarget | AiGoal::Follow => true,
            _ => false
        }
    }
}

fn get_default_goal(entity: Entity, world: &World) -> (AiGoal, Option<Entity>) {
    let ai = world.ecs().ais.get_or_err(entity);

    match ai.kind {
        AiKind::Wait => (AiGoal::DoNothing, None),
        AiKind::Wander => (AiGoal::Wander, None),
        AiKind::Follow => (AiGoal::Follow, world.player()),
        AiKind::Scavenge if ai.cond(AiProp::FoundItem, true) => {
            let items: Vec<Entity> = world.seen_entities(entity)
                .into_iter().filter(|&i| world.is_item(i))
                .collect();

            let chosen = entity.closest_entity(items, world);

            (AiGoal::GetItem, chosen)
        }
        AiKind::Scavenge => (AiGoal::FindItem, None),
        AiKind::SeekTarget => {
            match world.player() {
                Some(p) => {
                    if entity.can_see_other(p, world) {
                        (AiGoal::KillTarget, Some(p))
                    } else {
                        (AiGoal::FindTarget, Some(p))
                    }
                },
                None => (AiGoal::DoNothing, None),
            }
        },
    }
}

pub fn make_new_plan(entity: Entity, world: &World) -> (AiFacts, Option<Entity>, AiGoal) {
    let (goal, target) = get_default_goal(entity, world);
    debug_ecs!(world, entity, "New AI goal: {:?} on {:?}", goal, target);
    let desired = goal.get_end_state();
    (desired, target, goal)
}
