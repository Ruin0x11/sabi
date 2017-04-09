use std::cell::RefCell;

use goap::{Planner};
use rand::{self, Rng};

use actor::{Actor, ActorId};
use direction::Direction;
use action::Action;
use world::{World, Walkability};
use pathfinding::Path;
use drawcalls::Draw;

pub fn state_kill(id: &ActorId, state: &AiState) {
    let mut goal_c =  BTreeMap::new();
    goal_c.insert(AiProp::TargetDead, true);

    let goal = GoapState { facts: goal_c };
    *state.goal.borrow_mut() = goal;
    *state.target.borrow_mut() = Some(id.clone());
}

pub struct AiState {
    planner: AiPlanner,
    target: RefCell<Option<ActorId>>,
    memory: RefCell<AiMemory>,
    goal:   RefCell<AiMemory>,
}

type AiMemory = GoapState<AiProp, bool>;

impl AiState {
    pub fn new() -> Self {
        //TEMP: Figure out how to work with defaults.
        let mut facts = GoapFacts::new();
        facts.insert(AiProp::HealthLow, false);
        facts.insert(AiProp::HasTarget, false);
        facts.insert(AiProp::TargetVisible, false);
        facts.insert(AiProp::TargetDead, false);
        facts.insert(AiProp::NextToTarget, false);
        AiState {
            planner: make_planner(),
            target: RefCell::new(None),
            goal: RefCell::new(AiMemory {
                facts: facts.clone(),
            }),
            memory: RefCell::new(AiMemory {
                facts: facts,
            })
        }
    }
}

#[derive(Hash, Ord, PartialOrd, Eq, PartialEq, Debug, Clone)]
pub enum AiProp {
    HealthLow,
    HasTarget,
    TargetVisible,
    TargetDead,
    NextToTarget,
}

#[derive(Hash, Ord, PartialOrd, Eq, PartialEq, Debug, Clone)]
pub enum AiAction {
    Wander,
    MoveCloser,
    SwingAt,
    Run,
}

pub fn update_memory(actor: &Actor, world: &World) {
    let ref mut memory = actor.ai.memory.borrow_mut();
    memory.facts.insert(AiProp::HealthLow, actor.hp() < 50);
    match *actor.ai.target.borrow() {
        Some(ref id) => {
            let target = world.actor(id);
            memory.facts.insert(AiProp::HasTarget, true);
            memory.facts.insert(AiProp::TargetVisible, actor.can_see(&target.get_pos()));
            memory.facts.insert(AiProp::TargetDead, target.is_dead());
            memory.facts.insert(AiProp::NextToTarget, actor.get_pos().next_to(target.get_pos()));
        },
        None => {
            memory.facts.insert(AiProp::HasTarget, false);
            memory.facts.insert(AiProp::TargetVisible, false);
            memory.facts.insert(AiProp::TargetDead, false);
            memory.facts.insert(AiProp::NextToTarget, false);
        }
    }
}

pub fn update_goal(actor: &Actor, world: &World) {
    if actor.ai.planner.plan_found(&actor.ai.memory.borrow(), &actor.ai.goal.borrow()) {
        // The current plan has been finished. We need a new one.
        if let Some(id) = rand::thread_rng().choose(&actor.seen_actors(world)) {
            state_kill(id, &actor.ai);
        }
    }
}

pub fn choose_action(actor: &Actor, world: &World) -> Action {
    // TEMP: Just save the whole plan and only update when something interesting
    // happens
    let actions = actor.ai.planner.get_plan(&actor.ai.memory.borrow(), &actor.ai.goal.borrow());
    if let Some(action) = actions.first() {
        debug!(actor.logger, "the action: {:?}", action);
        match *action {
            AiAction::Wander => action_wander(actor, world),
            AiAction::MoveCloser => action_move_closer(actor, world),
            AiAction::SwingAt => action_swing_at(actor, world),
            AiAction::Run => action_run(actor, world),
        }
    } else {
        warn!(actor.logger, "I can't figure out what to do!");
        Action::Wait
    }
}

fn action_wander(_actor: &Actor, _world: &World) -> Action {
    Action::Move(Direction::choose8())
}

fn action_swing_at(actor: &Actor, _world: &World) -> Action {
    Action::SwingAt(actor.ai.target.borrow().unwrap())
}

fn action_move_closer(actor: &Actor, world: &World) -> Action {
    let target = world.actor(&actor.ai.target.borrow().unwrap());
    assert!(!target.is_dead(), "Target is already dead!");

    let my_pos = actor.get_pos();
    let target_pos = target.get_pos();

    assert!(actor.can_see(&target_pos), "Actor can't see target!");

    // Am I right next to the target?
    match Direction::from_neighbors(my_pos, target_pos) {
        Some(dir) => return Action::Move(dir),
        None      => (),
    }

    let mut path = Path::find(my_pos, target_pos, &world, Walkability::MonstersBlocking);

    debug!(actor.logger, "My: {} target: {}, path: {:?}", my_pos, target_pos, path);

    if path.len() == 0 {
        // TODO: Lost sight of target.
        return Action::Wait;
    }

    let next_pos = path.next().unwrap();

    for pt in path {
        world.draw_calls.push(Draw::Point(pt.x, pt.y));
    }

    match Direction::from_neighbors(my_pos, next_pos) {
        Some(dir) => Action::Move(dir),
        None      => panic!("Can't traverse path: {} {}", my_pos, next_pos),
    }
}

fn action_run(_actor: &Actor, _world: &World) -> Action {
    Action::Move(Direction::choose8())
}

use std::collections::{BTreeMap, HashMap};
use goap::*;
use stats::properties::PropType;

type AiPlanner = GoapPlanner<AiProp, bool, AiAction>;

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
