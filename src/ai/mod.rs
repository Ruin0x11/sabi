mod action;
mod goal;
mod sensors;
mod trigger;

use self::action::*;
use self::goal::*;
use self::sensors::*;
pub use self::goal::AiKind;
pub use self::trigger::AiTrigger;

use std::cell::RefCell;
use std::collections::HashMap;

use calx_ecs::Entity;
use goap::*;

use logic::Action;
use ai::sensors::Sensor;
use ecs::traits::ComponentQuery;
use world::traits::Query;
use world::World;

#[derive(Eq, PartialEq, Clone, Debug, Serialize, Deserialize)]
pub enum Disposition {
    Friendly,
    Enemy,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Ai {
    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    #[serde(default = "make_planner")]
    planner: AiPlanner,

    kind: AiKind,

    target: RefCell<Option<Entity>>,
    memory: RefCell<AiMemory>,
    goal: RefCell<AiMemory>,
    next_action: RefCell<Option<AiAction>>,

    // TODO: Keep track of other entities turned enemies, such as due to too much friendly fire
    pub disposition: Disposition,

    pub last_goal: RefCell<AiGoal>,
}

pub type AiMemory = GoapState<AiProp, bool>;
pub type AiFacts = GoapFacts<AiProp, bool>;

impl Ai {
    pub fn new(kind: AiKind) -> Self {
        //TEMP: Figure out how to work with defaults.
        let facts = default_ai_facts();
        Ai {
            planner: make_planner(),
            target: RefCell::new(None),
            goal: RefCell::new(AiMemory { facts: facts.clone() }),
            memory: RefCell::new(AiMemory { facts: facts }),
            disposition: Disposition::Friendly,
            kind: kind,

            next_action: RefCell::new(None),
            last_goal: RefCell::new(AiGoal::DoNothing),
        }
    }

    pub fn cond(&self, prop: AiProp, val: bool) -> bool {
        self.memory.borrow().facts.get(&prop).map_or(false, |f| *f == val)
    }

    pub fn get_plan(&self) -> Result<Vec<AiAction>, AiMemory> {
        self.planner.get_plan(
            &self.memory.borrow(),
            &self.goal.borrow(),
        )
    }

    pub fn get_next_action(&self) -> Option<AiAction> {
        self.get_plan().unwrap_or(Vec::new()).first().cloned()
    }

    pub fn goal_finished(&self) -> bool {
        let invalid = self.is_state_invalid();
        let no_possible_action = self.next_action.borrow().is_none();
        invalid || no_possible_action
    }

    fn is_state_invalid(&self) -> bool {
        if self.last_goal.borrow().requires_target() {
            if self.target.borrow().is_none() {
                return true
            }
        }

        false
    }

    pub fn add_memory(&mut self, trigger: AiTrigger) {
    }

    pub fn debug_info(&self) -> String {
        format!("target: {:?}, goal: {:?}", self.target.borrow(), self.last_goal.borrow())
    }
}

thread_local! {
    static SENSORS: HashMap<AiProp, Sensor> = sensors::make_sensors();
}

pub fn run(entity: Entity, world: &World) -> Option<Action> {
    assert!(
        !world.is_player(entity),
        "Tried running AI on current player!"
    );

    if !world.ecs().ais.has(entity) {
        assert!(
            !world.turn_order().contains(entity),
            "Entity without ai in turn order!"
        );
        return None;
    }

    check_target(entity, world);
    update_goal(entity, world);
    update_memory(entity, world);
    let action = choose_action(entity, world);

    Some(action)
}

fn check_target(entity: Entity, world: &World) {
    // The entity reference could go stale, so make sure it isn't.
    // TODO: Should this have to happen every time an entity reference is held
    // by something?
    let ai = world.ecs().ais.get_or_err(entity);

    let mut target = ai.target.borrow_mut();
    let dead = target.map_or(true, |t| world.position(t).is_none());
    let removed = target.map_or(true, |t| !world.ecs().contains(t));
    let in_inventory = target.map_or(false, |t| world.entities_in(entity).contains(&t));
    // debug_ecs!(entity, world, "Target: {:?} dead {} removed {} in inv {}", target, dead, removed, in_inventory);
    if target.is_some() && (dead || removed) && !in_inventory {
        *target = None;
    }
}

fn update_goal(entity: Entity, world: &World) {
    let ai = world.ecs().ais.get_or_err(entity);

    if ai.goal_finished() {
        debug_ecs!(world, entity, "Last goal finished.");
        let (desired, target, goal_kind) = make_new_plan(entity, world);

        let goal = GoapState { facts: desired };
        *ai.last_goal.borrow_mut() = goal_kind;
        *ai.goal.borrow_mut() = goal;
        *ai.target.borrow_mut() = target;
    }
}

fn update_memory(entity: Entity, world: &World) {
    let ai = world.ecs().ais.get_or_err(entity);
    let wants_to_know = all_props();
    let mut new_memory = AiMemory { facts: GoapFacts::new() };

    for fact in wants_to_know.iter() {
        SENSORS.with(|s| {
            let sensor = match s.get(fact) {
                Some(f) => f,
                None    => return,
            };
            let result = (sensor.callback)(world, entity, ai);
            // debug_ecs!(world, entity, "{:?}, {}", fact, result);
            new_memory.facts.insert(fact.clone(), result);
        });
    }

    let stale = {
        let memory = ai.memory.borrow();
        *memory != new_memory
    };

    debug_ecs!(world, entity, "Target: {:?}", ai.target.borrow());

    if stale {
        debug_ecs!(world, entity, "Regenerating AI cache! {:?}", ai.memory.borrow());
        // make sure the memory is fresh before picking an action
        *ai.memory.borrow_mut() = new_memory;

        let next_action = ai.get_next_action();
        debug_ecs!(world, entity, "Do thing: {:?}", next_action);
        *ai.next_action.borrow_mut() = next_action;
    }
}

type AiPlanner = GoapPlanner<AiProp, bool, AiAction>;

// FIXME: ugh?
pub fn make_planner() -> AiPlanner {
    let mut actions = HashMap::new();
    let mut effects = GoapEffects::new(100);
    effects.set_precondition(AiProp::Moving, false);
    effects.set_postcondition(AiProp::Moving, true);
    effects.set_postcondition(AiProp::HasTarget, true);
    effects.set_postcondition(AiProp::TargetVisible, true);
    actions.insert(AiAction::Wander, effects);

    let mut effects = GoapEffects::new(12);
    effects.set_precondition(AiProp::HasTarget, true);
    effects.set_precondition(AiProp::OnTopOfTarget, true);
    effects.set_precondition(AiProp::TargetInInventory, false);
    effects.set_postcondition(AiProp::TargetVisible, true);
    effects.set_postcondition(AiProp::TargetInInventory, true);
    actions.insert(AiAction::PickupItem, effects);

    let mut effects = GoapEffects::new(10);
    effects.set_precondition(AiProp::HasTarget, true);
    effects.set_precondition(AiProp::OnTopOfTarget, false);
    effects.set_postcondition(AiProp::NextToTarget, true);
    effects.set_postcondition(AiProp::OnTopOfTarget, true);
    effects.set_postcondition(AiProp::TargetVisible, true);
    actions.insert(AiAction::MoveCloser, effects);

    let mut effects = GoapEffects::new(9);
    effects.set_precondition(AiProp::HasTarget, true);
    effects.set_precondition(AiProp::TargetVisible, true);
    effects.set_precondition(AiProp::NextToTarget, true);
    effects.set_precondition(AiProp::CanDoMelee, true);
    effects.set_precondition(AiProp::TargetDead, false);
    effects.set_postcondition(AiProp::TargetDead, true);
    actions.insert(AiAction::SwingAt, effects);

    let mut effects = GoapEffects::new(8);
    effects.set_precondition(AiProp::HasTarget, true);
    effects.set_precondition(AiProp::TargetVisible, true);
    effects.set_precondition(AiProp::NextToTarget, false);
    effects.set_precondition(AiProp::CanDoRanged, true);
    effects.set_precondition(AiProp::TargetDead, false);
    effects.set_postcondition(AiProp::TargetDead, true);
    actions.insert(AiAction::ShootAt, effects);

    let mut effects = GoapEffects::new(2);
    effects.set_precondition(AiProp::HealthLow, true);
    effects.set_postcondition(AiProp::HealthLow, false);
    actions.insert(AiAction::RunAway, effects);

    GoapPlanner { actions: actions }
}
