mod action;
mod sensors;

use std::cell::RefCell;

use calx_ecs::Entity;

use logic::Action;
use ai::sensors::{Sensor};
use world::traits::{ComponentQuery, Query};
use world::EcsWorld;

#[derive(Eq, PartialEq, Clone, Debug, Serialize, Deserialize)]
pub enum Disposition {
    Friendly,
    Enemy,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Ai {
    #[serde(default="make_planner")]
    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    planner: AiPlanner,

    target: RefCell<Option<Entity>>,
    memory: RefCell<AiMemory>,
    goal:   RefCell<AiMemory>,

    pub disposition: Disposition,
}

pub fn state_kill(target: Entity, state: &Ai) {
    let mut goal_c =  BTreeMap::new();
    goal_c.insert(AiProp::TargetDead, true);

    let goal = GoapState { facts: goal_c };
    *state.goal.borrow_mut() = goal;
    *state.target.borrow_mut() = Some(target);
}

type AiMemory = GoapState<AiProp, bool>;

impl Ai {
    pub fn new() -> Self {
        //TEMP: Figure out how to work with defaults.
        let mut facts = GoapFacts::new();
        facts.insert(AiProp::HealthLow, false);
        facts.insert(AiProp::HasTarget, false);
        facts.insert(AiProp::TargetVisible, false);
        facts.insert(AiProp::TargetDead, false);
        facts.insert(AiProp::NextToTarget, false);
        Ai {
            planner: make_planner(),
            target: RefCell::new(None),
            goal: RefCell::new(AiMemory {
                facts: facts.clone(),
            }),
            memory: RefCell::new(AiMemory {
                facts: facts,
            }),
            disposition: Disposition::Friendly,
        }
    }

    pub fn get_plan(&self) -> Vec<AiAction> {
        self.planner.get_plan(&self.memory.borrow(), &self.goal.borrow())
    }

    pub fn goal_finished(&self) -> bool {
        self.get_plan().is_empty()
    }
}

#[derive(Serialize, Deserialize, Hash, Ord, PartialOrd, Eq, PartialEq, Debug, Clone)]
pub enum AiProp {
    HealthLow,
    HasTarget,
    TargetVisible,
    TargetDead,
    NextToTarget,
}

#[derive(Serialize, Deserialize, Hash, Ord, PartialOrd, Eq, PartialEq, Debug, Clone)]
pub enum AiAction {
    Wander,
    MoveCloser,
    SwingAt,
    Run,
}

thread_local! {
    static SENSORS: HashMap<AiProp, Sensor> = sensors::make_sensors();
}

pub fn run(entity: Entity, world: &EcsWorld) -> Action {
    assert!(!world.is_player(entity), "Tried running AI on current player!");

    check_target(entity, world);
    update_goal(entity, world);
    update_memory(entity, world);
    choose_action(entity, world)
}

fn check_target(entity: Entity, world: &EcsWorld) {
    // The entity reference could go stale, so make sure it isn't.
    // TODO: Should this have to happen every time an entity reference is held
    // by something?
    let ai = world.ecs().ais.get_or_err(entity);

    let mut target = ai.target.borrow_mut();
    let dead = target.map_or(true, |t| world.position(t).is_none());
    let removed = target.map_or(true, |t| !world.ecs().contains(t));
    if target.is_some() && (dead || removed) {
        *target = None;
    }
}

fn update_memory(entity: Entity, world: &EcsWorld) {
    let ai = world.ecs().ais.get_or_err(entity);
    let ref mut memory = ai.memory.borrow_mut();
    let wants_to_know = vec![AiProp::HasTarget,
                             AiProp::TargetVisible,
                             AiProp::TargetDead,
                             AiProp::NextToTarget,
                             AiProp::HealthLow];
    for fact in wants_to_know.iter() {
        SENSORS.with(|s| {
            let sensor = s.get(fact).unwrap();
            let result = (sensor.callback)(world, &entity, ai);
            // debug_ecs!(world, entity, "{:?}, {}", fact, result);
            memory.facts.insert(fact.clone(), result);
        });
    }
}

fn update_goal(entity: Entity, world: &EcsWorld) {
    let ai = world.ecs().ais.get_or_err(entity);

    if ai.goal_finished() {
        // TODO: Determine a new plan.
        // if let Some(target) = rand::thread_rng().choose(&world.seen_entities(entity)) {
        //     state_kill(*target, ai);
        // }
        world.player().map(|p| {
            if world.seen_entities(entity).contains(&p) {
                state_kill(p, ai);
            }
        });
    }
}

fn choose_action(entity: Entity, world: &EcsWorld) -> Action {
    // TEMP: Just save the whole plan and only update when something interesting
    // happens
    let ai = world.ecs().ais.get_or_err(entity);
    let actions = ai.get_plan();
    if let Some(action) = actions.first() {
        match *action {
            AiAction::Wander => action::wander(entity, world),
            AiAction::MoveCloser => action::move_closer(entity, world),
            AiAction::SwingAt => action::swing_at(entity, world),
            AiAction::Run => action::run_away(entity, world),
        }
    } else {
        warn_ecs!(world, entity, "I can't figure out what to do!");
        action::wander(entity, world)
    }
}

use std::collections::{BTreeMap, HashMap};
use goap::*;

type AiPlanner = GoapPlanner<AiProp, bool, AiAction>;

// FIXME: ugh?
pub fn make_planner() -> AiPlanner {
    let mut actions = HashMap::new();
    let mut effects = GoapEffects::new(100);
    effects.set_precondition(AiProp::TargetVisible, false);
    actions.insert(AiAction::Wander, effects);

    let mut effects = GoapEffects::new(10);
    effects.set_precondition(AiProp::HasTarget, true);
    effects.set_precondition(AiProp::TargetVisible, true);
    effects.set_precondition(AiProp::NextToTarget, false);
    effects.set_precondition(AiProp::TargetDead, false);
    effects.set_postcondition(AiProp::NextToTarget, true);
    actions.insert(AiAction::MoveCloser, effects);

    let mut effects = GoapEffects::new(9);
    effects.set_precondition(AiProp::HasTarget, true);
    effects.set_precondition(AiProp::TargetVisible, true);
    effects.set_precondition(AiProp::NextToTarget, true);
    effects.set_precondition(AiProp::TargetDead, false);
    effects.set_postcondition(AiProp::TargetDead, true);
    actions.insert(AiAction::SwingAt, effects);

    let mut effects = GoapEffects::new(2);
    effects.set_precondition(AiProp::HealthLow, true);
    effects.set_postcondition(AiProp::HealthLow, false);
    actions.insert(AiAction::Run, effects);
    GoapPlanner {
        actions: actions,
    }
}
