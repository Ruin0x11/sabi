use calx_ecs::Entity;

use ai::*;
use ecs::traits::ComponentQuery;
use logic::entity::EntityQuery;
use world::World;
use world::traits::Query;

#[derive(Eq, PartialEq, Clone, Copy, Debug, Serialize, Deserialize)]
pub enum AiKind {
    Wait,
    SeekTarget,
    Follow,
    Wander,
    Scavenge,
    Guard,
}

impl AiKind {
    pub fn on_goal(&self, goal: AiGoal, entity: Entity, world: &mut World) {
        match *self {
            AiKind::Guard => {
                match goal {
                    AiGoal::KillTarget => {
                        format_mes!(world, entity, "%u: Scum!");
                    },
                    _ => (),
                }
            },
            _ => (),
        }
    }
}

#[derive(Eq, PartialEq, Clone, Copy, Debug, Serialize, Deserialize)]
pub enum AiGoal {
    FindTarget,
    KillTarget,
    Follow,
    Wander,
    EscapeDanger,
    Guard,

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
            AiGoal::Guard => vec![(AiProp::AtPosition, true), (AiProp::Moving, false)],

            AiGoal::EscapeDanger => vec![(AiProp::Exists, false)],

            AiGoal::FindItem => vec![(AiProp::FoundItem, true)],
            AiGoal::GetItem => vec![(AiProp::TargetInInventory, true)],

            AiGoal::DoNothing => vec![(AiProp::Exists, false)],
        }
    }

    pub fn requires_target(&self) -> bool {
        match *self {
            AiGoal::GetItem | AiGoal::FindTarget | AiGoal::KillTarget | AiGoal::Follow => true,
            _ => false,
        }
    }

    pub fn requires_position(&self) -> bool {
        match *self {
            AiGoal::Guard => true,
            _ => false,
        }
    }
}

fn hostile_entity(entity: Entity, world: &World) -> Option<Entity> {
    world.seen_entities(entity)
         .into_iter()
         .find(|e| e.is_hostile(entity, world))
}

fn get_default_goal(entity: Entity, world: &World) -> (AiGoal, Option<Target>) {
    let ai_compo = world.ecs().ais.get_or_err(entity);
    let ai = &ai_compo.data;

    match ai_compo.kind {
        AiKind::Follow => goal_follow(entity, world),
        AiKind::Guard => goal_guard(entity, world),
        AiKind::SeekTarget => goal_seek_target(entity, world),

        AiKind::Wait => (AiGoal::DoNothing, None),
        AiKind::Wander => (AiGoal::Wander, None),
        AiKind::Scavenge if ai.cond(AiProp::FoundItem, true) => goal_scavenge(entity, world),
        AiKind::Scavenge => (AiGoal::FindItem, None),
    }
}

fn attack_target(entity: Entity) -> Target {
    Target {
        entity: entity,
        priority: 100,
        kind: TargetKind::Attack,
    }
}

fn goal_follow(entity: Entity, world: &World) -> (AiGoal, Option<Target>) {
    if let Some(hostile) = hostile_entity(entity, world) {
        return (AiGoal::KillTarget, Some(attack_target(hostile)));
    }

    (AiGoal::Follow,
     world.player().map(|p| {
        Target {
            entity: p,
            priority: 1000,
            kind: TargetKind::Other,
        }
    }))
}

fn goal_guard(entity: Entity, world: &World) -> (AiGoal, Option<Target>) {
    if let Some(hostile) = hostile_entity(entity, world) {
        return (AiGoal::KillTarget, Some(attack_target(hostile)));
    }

    (AiGoal::Guard, None)
}

fn goal_scavenge(entity: Entity, world: &World) -> (AiGoal, Option<Target>) {

    let items: Vec<Entity> = world.seen_entities(entity)
                                  .into_iter()
                                  .filter(|&i| world.is_item(i))
                                  .collect();

    let chosen = entity.closest_entity(items, world);

    (AiGoal::GetItem,
     chosen.map(|t| {
        Target {
            entity: t,
            priority: 100,
            kind: TargetKind::Pickup,
        }
    }))
}

fn goal_seek_target(entity: Entity, world: &World) -> (AiGoal, Option<Target>) {
    match world.player() {
        Some(player) => {
            if entity.can_see_other(player, world) {
                (AiGoal::KillTarget, Some(attack_target(player)))
            } else if let Some(hostile) = hostile_entity(entity, world) {
                return (AiGoal::KillTarget, Some(attack_target(hostile)));
            } else {
                (AiGoal::FindTarget, Some(attack_target(player)))
            }
        },
        None => (AiGoal::DoNothing, None),
    }

}


pub fn make_new_plan(entity: Entity, world: &World) -> (AiFacts, Option<Target>, AiGoal) {
    let (goal, target) = get_default_goal(entity, world);
    debug_ecs!(world, entity, "New AI goal: {:?} on {:?}", goal, target);
    let desired = goal.get_end_state();
    (desired, target, goal)
}
