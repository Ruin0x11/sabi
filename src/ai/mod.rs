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

use ai::sensors::Sensor;
use ecs::traits::ComponentQuery;
use logic::Action;
use point::Point;
use world::World;
use world::traits::Query;
use macros::{Getter, TomlInstantiate};
use util;
use toml;

#[derive(Eq, PartialEq, Clone, Debug, Serialize, Deserialize)]
pub enum Attitude {
    Friendly,
    Neutral,
    Hostile,
}

make_getter!(Ai {
                 kind: AiKind,

                 data: AiData,
             });

impl Default for Ai {
    fn default() -> Self {
        Ai::new(AiKind::Guard)
    }
}

impl Ai {
    pub fn new(kind: AiKind) -> Ai {
        Ai {
            kind: kind,
            data: AiData::new(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AiData {
    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    #[serde(default = "planner_from_toml")]
    planner: AiPlanner,

    important_pos: RefCell<Option<Point>>,
    targets: RefCell<Targets>,
    memory: RefCell<AiMemory>,
    goal: RefCell<AiMemory>,
    next_action: RefCell<Option<AiAction>>,
    triggers: RefCell<Vec<AiTrigger>>,

    pub last_goal: RefCell<AiGoal>,
}

pub type AiMemory = GoapState<AiProp, bool>;
pub type AiFacts = GoapFacts<AiProp, bool>;

impl AiData {
    pub fn new() -> Self {
        //TEMP: Figure out how to work with defaults.
        let facts = default_ai_facts();
        AiData {
            planner: planner_from_toml(),

            important_pos: RefCell::new(None),
            targets: RefCell::new(Targets::new()),
            goal: RefCell::new(AiMemory { facts: facts.clone() }),
            memory: RefCell::new(AiMemory { facts: facts }),

            next_action: RefCell::new(None),
            triggers: RefCell::new(Vec::new()),
            last_goal: RefCell::new(AiGoal::DoNothing),
        }
    }

    pub fn cond(&self, prop: AiProp, val: bool) -> bool {
        self.memory
            .borrow()
            .facts
            .get(&prop)
            .map_or(false, |f| *f == val)
    }

    pub fn get_plan(&self) -> Result<Vec<AiAction>, AiMemory> {
        self.planner
            .get_plan(&self.memory.borrow(), &self.goal.borrow())
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
            if self.targets.borrow().is_empty() {
                return true;
            }
        }

        if self.last_goal.borrow().requires_position() {
            if self.important_pos.borrow().is_none() {
                return true;
            }
        }

        false
    }

    pub fn add_memory(&self, trigger: AiTrigger) {
        self.triggers.borrow_mut().push(trigger);
    }

    pub fn debug_info(&self) -> String {
        format!("target: {:?}, goal: {:?}",
                self.targets.borrow().peek(),
                self.last_goal.borrow())
    }
}

thread_local! {
    static SENSORS: HashMap<AiProp, Sensor> = sensors::make_sensors();
}

pub fn run(entity: Entity, world: &World) -> Option<Action> {
    assert!(!world.is_player(entity), "Tried running AI on current player!");

    if !world.ecs().ais.has(entity) {
        assert!(!world.turn_order().contains(entity), "Entity without ai in turn order!");
        return None;
    }

    check_target(entity, world);
    update_goal(entity, world);
    check_triggers(entity, world);
    update_memory(entity, world);
    let action = choose_action(entity, world);

    Some(action)
}

fn check_target(entity: Entity, world: &World) {
    // The entity reference could go stale, so make sure it isn't.
    // TODO: Should this have to happen every time an entity reference is held
    // by something?
    let mut target_is_invalid = false;
    let ai = &world.ecs().ais.get_or_err(entity).data;

    {
        let target = ai.targets.borrow();
        let dead = target.peek()
                         .map_or(true, |t| world.position(t.entity).is_none());
        let removed = target.peek()
                            .map_or(true, |t| !world.ecs().contains(t.entity));
        let in_inventory =
            target.peek()
                  .map_or(false, |t| world.entities_in(entity).contains(&t.entity));
        // debug_ecs!(entity, world, "Target: {:?} dead {} removed {} in inv {}", target, dead, removed, in_inventory);
        if target.peek().is_some() && (dead || removed) && !in_inventory {
            target_is_invalid = true;
        }
    }

    if target_is_invalid {
        ai.targets.borrow_mut().pop();
    }

    if ai.important_pos.borrow().is_none() {
        *ai.important_pos.borrow_mut() = world.position(entity)
    }
}

fn check_triggers(entity: Entity, world: &World) {
    let ai = world.ecs().ais.get_or_err(entity);

    if let Some((goal, target)) = ai.kind.check_triggers(entity, world) {
        set_goal(entity, world, goal.get_end_state(), target, goal);
    }

    ai.data.triggers.borrow_mut().clear();
}

fn update_goal(entity: Entity, world: &World) {
    let ai = &world.ecs().ais.get_or_err(entity).data;

    if ai.goal_finished() {
        debug_ecs!(world, entity, "Last goal finished.");
        let (desired, target, goal_kind) = make_new_plan(entity, world);

        set_goal(entity, world, desired, target, goal_kind);
    }
}

fn set_goal(entity: Entity,
            world: &World,
            desired: AiFacts,
            target: Option<Target>,
            goal_kind: AiGoal) {
    let ai = &world.ecs().ais.get_or_err(entity).data;

    let goal = GoapState { facts: desired };
    *ai.last_goal.borrow_mut() = goal_kind;
    *ai.goal.borrow_mut() = goal;

    if let Some(t) = target {
        ai.targets.borrow_mut().set_sole(t);
    }

    // ai.kind.on_goal(goal_kind, entity, world);
}

fn update_memory(entity: Entity, world: &World) {
    let ai = world.ecs().ais.get_or_err(entity);
    let wants_to_know = all_props();
    let mut new_memory = AiMemory { facts: GoapFacts::new() };

    for fact in wants_to_know.iter() {
        SENSORS.with(|s| {
            let sensor = match s.get(fact) {
                Some(f) => f,
                None => return,
            };
            let result = (sensor.callback)(world, entity, ai);
            // debug_ecs!(world, entity, "{:?}, {}", fact, result);
            new_memory.facts.insert(fact.clone(), result);
        });
    }

    let stale = {
        let memory = ai.data.memory.borrow();
        *memory != new_memory
    };

    if stale {
        debug_ecs!(world, entity, "Regenerating AI cache! {:?}", ai.data.memory.borrow());
        debug_ecs!(world, entity, "Target: {:?}", ai.data.targets.borrow().peek());

        // make sure the memory is fresh before picking an action
        *ai.data.memory.borrow_mut() = new_memory;

        let next_action = ai.data.get_next_action();
        debug_ecs!(world, entity, "Do thing: {:?}", next_action);
        *ai.data.next_action.borrow_mut() = next_action;
    }
}

type AiPlanner = GoapPlanner<AiProp, bool, AiAction>;

// TODO: Reverse priorities, so larger priorities are more important
fn planner_from_toml() -> AiPlanner {
    let mut actions = HashMap::new();
    let value = util::toml::toml_value_from_file("./data/actions.toml");
    if let toml::Value::Array(ref array) = value["action"] {
        for action in array.iter() {
            // Parse string as enum
            let name = action["name"]
                .clone()
                .try_into::<String>()
                .unwrap()
                .parse::<AiAction>()
                .unwrap();
            let cost = action["cost"].clone().try_into::<u32>().unwrap();
            let mut effects = GoapEffects::new(cost);

            // TODO: crashes if action.pre or action.post don't exist
            // Read preconditions
            if let toml::Value::Table(ref pre_table) = action["pre"] {
                for (pre_name, pre_value) in pre_table.iter() {
                    let key = pre_name.parse::<AiProp>().unwrap();
                    let value = pre_value.clone().try_into::<bool>().unwrap();
                    effects.set_precondition(key, value);
                }
            }

            // Read postconditions
            if let toml::Value::Table(ref post_table) = action["post"] {
                for (post_name, post_value) in post_table.iter() {
                    let key = post_name.parse::<AiProp>().unwrap();
                    let value = post_value.clone().try_into::<bool>().unwrap();
                    effects.set_postcondition(key, value);
                }
            }

            actions.insert(name, effects);
        }
    }

    GoapPlanner { actions: actions }
}

use std::collections::BinaryHeap;
use std::cmp::Ordering;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum TargetKind {
    Attack,
    Pickup,
    Other,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Target {
    entity: Entity,
    priority: u32,
    kind: TargetKind,
}

impl Ord for Target {
    fn cmp(&self, other: &Target) -> Ordering {
        self.priority.cmp(&other.priority)
        //.then_with(|| self.position.cmp(&other.position))
    }
}

impl PartialOrd for Target {
    fn partial_cmp(&self, other: &Target) -> Option<Ordering> {
        Some(other.cmp(self))
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Targets {
    targets: BinaryHeap<Target>,
}

impl Targets {
    pub fn new() -> Self {
        Targets { targets: BinaryHeap::new() }
    }

    pub fn is_empty(&self) -> bool {
        self.targets.is_empty()
    }

    pub fn peek(&self) -> Option<&Target> {
        self.targets.peek()
    }

    pub fn pop(&mut self) -> Option<Target> {
        self.targets.pop()
    }

    pub fn push(&mut self, target: Target) {
        self.targets.push(target);
    }

    pub fn set_sole(&mut self, target: Target) {
        self.targets = BinaryHeap::new();
        self.targets.push(target);
    }
}
