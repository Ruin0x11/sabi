use calx_ecs::Entity;

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
}

#[derive(Eq, PartialEq, Clone, Debug, Serialize, Deserialize)]
pub enum AiGoal {
    FindTarget,
    KillTarget,
    Follow,
    Wander,
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
        // TODO: instead make the "health low" things triggers for entering the new goal of "run //
        // away and heal"
        match *self {
            AiGoal::FindTarget => vec![(AiProp::TargetVisible, true), (AiProp::HealthLow, false)],
            AiGoal::KillTarget => vec![(AiProp::TargetDead, true), (AiProp::HealthLow, false)],
            AiGoal::Follow => vec![(AiProp::NextToTarget, true)],
            AiGoal::Wander => vec![(AiProp::Moving, true)],
            AiGoal::DoNothing => vec![(AiProp::Exists, false)],
        }
    }
}

fn get_goal(entity: Entity, world: &World) -> (AiGoal, Option<Entity>) {
    let ai = world.ecs().ais.get_or_err(entity);

    match ai.kind {
        AiKind::Wait => (AiGoal::DoNothing, None),
        AiKind::Wander => (AiGoal::Wander, None),
        AiKind::Follow => (AiGoal::Follow, world.player()),
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

pub fn make_new_plan(entity: Entity, world: &World) -> (AiFacts, Option<Entity>) {
    let (goal, target) = get_goal(entity, world);
    let desired = goal.get_end_state();
    (desired, target)
}
