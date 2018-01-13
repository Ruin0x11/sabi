mod action;
mod goal;
mod sensors;
mod trigger;

use self::action::*;
use self::goal::*;
use self::sensors::*;
pub use self::goal::AiKind;
pub use self::trigger::AiTrigger;

use std::cell::{Cell, RefCell};
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

make_global!(AI_PLANNER, AiPlanner, planner_from_toml());

pub fn reload_planner() {
    instance::with_mut(|planner| *planner = planner_from_toml());
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AiData {
    important_pos: RefCell<Option<Point>>,
    targets: RefCell<Targets>,
    memory: RefCell<AiMemory>,
    goal: RefCell<AiMemory>,
    next_action: RefCell<Option<AiAction>>,
    target_was_switched: Cell<bool>,
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
            important_pos: RefCell::new(None),
            targets: RefCell::new(Targets::new()),
            goal: RefCell::new(AiMemory { facts: facts.clone() }),
            memory: RefCell::new(AiMemory { facts: facts }),

            next_action: RefCell::new(None),
            target_was_switched: Cell::new(false),
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
        instance::with(|planner| planner.get_plan(&self.memory.borrow(), &self.goal.borrow()))
    }

    pub fn get_next_action(&self) -> Option<AiAction> {
        self.get_plan().unwrap_or(Vec::new()).first().cloned()
    }

    pub fn goal_finished(&self) -> bool {
        let invalid = self.is_state_invalid();
        let no_possible_action = self.next_action.borrow().is_none();

        println!("inv: {}, no: {}", invalid, no_possible_action);
        println!("{:?}", self.get_next_action());
        invalid || no_possible_action
    }

    fn is_state_invalid(&self) -> bool {
        if self.last_goal.borrow().requires_target() {
            if self.targets.borrow().is_empty() {
                return true;
            }

            if self.targets.borrow().peek().unwrap().entity.is_none() {
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
        let mut senses = String::new();
        for (fact, truth) in self.memory.borrow().facts.iter() {
            if *truth {
                senses = format!("{}{:?}\n", senses, fact);
            }
        }
        format!("targets:\n{}\ngoal:{:?}\nplan:{:?}\nnext:{:?}\nsenses:{}",
                self.targets.borrow(),
                self.goal.borrow(),
                self.get_plan(),
                self.get_next_action(),
                senses)
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
        // make sure the memory is fresh before picking an action
        *ai.data.memory.borrow_mut() = new_memory;

        update_next_action(entity, world);
    }
}

pub fn update_next_action(entity: Entity, world: &World) {
    let ai = world.ecs().ais.get_or_err(entity);
    let next_action = ai.data.get_next_action();
    debug_ecs!(world, entity, "Do thing: {:?}", next_action);
    *ai.data.next_action.borrow_mut() = next_action;
}

// FIXME: increase speed
fn check_target(entity: Entity, world: &World) {
    // The entity reference could go stale, so make sure it isn't.
    // TODO: Should this have to happen every time an entity reference is held
    // by something?
    let mut target_is_invalid = false;
    let ai = &world.ecs().ais.get_or_err(entity).data;

    {
        let target = ai.targets.borrow();
        if !ai.targets.borrow().is_empty() && ai.targets.borrow().peek().unwrap().entity.is_some() {
            let entity = ai.targets.borrow().peek().unwrap().entity.unwrap();
            let dead = target.peek()
                             .map_or(true, |t| world.position(entity).is_none());
            let removed = target.peek()
                                .map_or(true, |t| !world.ecs().contains(entity));
            let in_inventory =
                target.peek()
                      .map_or(false, |t| world.entities_in(entity).contains(&entity));
            // debug_ecs!(world,
            //            entity,
            //            "Target: {:?} dead {} removed {} in inv {}",
            //            target,
            //            dead,
            //            removed,
            //            in_inventory);
            if (dead || removed) && !in_inventory {
                target_is_invalid = true;
            }
        }
    }

    if target_is_invalid {
        finish_target(entity, world);
    }

    if ai.important_pos.borrow().is_none() {
        *ai.important_pos.borrow_mut() = world.position(entity)
    }
}

fn check_triggers(entity: Entity, world: &World) {
    let ai = world.ecs().ais.get_or_err(entity);

    if let Some((goal, target)) = ai.kind.check_triggers(entity, world) {
        set_goal(entity, world, goal.get_end_state(), target);
    }

    ai.data.triggers.borrow_mut().clear();
}

fn update_goal(entity: Entity, world: &World) {
    let ai = &world.ecs().ais.get_or_err(entity).data;

    if ai.goal_finished() {
        debug_ecs!(world, entity, "Last goal finished: {:?}.", ai.last_goal.borrow());

        // Target finished, stop tracking it.
        finish_target(entity, world);
    }
}

fn add_target(target: Target, entity: Entity, world: &World) {
    let ai = &world.ecs().ais.get_or_err(entity).data;
    ai.targets.borrow_mut().push(target);
    debug_ecs!(world, entity, "Target pushed: {:?}", target);

    on_target_switch(entity, world);
}

fn finish_target(entity: Entity, world: &World) {
    let ai = &world.ecs().ais.get_or_err(entity).data;
    let target = ai.targets.borrow_mut().pop();
    debug_ecs!(world, entity, "Target popped: {:?}", target);

    on_target_switch(entity, world);
}

fn on_target_switch(entity: Entity, world: &World) {
    let (desired, target) = make_new_plan(entity, world);
    debug_ecs!(world, entity, "AI target was changed! {:?}", target);
    set_goal(entity, world, desired, target);

    // to avoid borrowing next_action twice

    let ai = &world.ecs().ais.get_or_err(entity).data;
    ai.target_was_switched.set(true);
}

fn set_goal(entity: Entity, world: &World, desired: AiFacts, target_opt: Option<Target>) {
    let ai = &world.ecs().ais.get_or_err(entity).data;

    let goal = GoapState { facts: desired };
    *ai.goal.borrow_mut() = goal;

    if let Some(target) = target_opt {
        //debug_ecs!(world, entity, "Setting last goal to {:?}, {:?}", target.goal, target);
        *ai.last_goal.borrow_mut() = target.goal;
        ai.targets.borrow_mut().push(target);
    }

    // ai.kind.on_goal(goal_kind, entity, world);
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
pub struct Target {
    entity: Option<Entity>,
    priority: u32,
    goal: AiGoal,
}

impl Target {
    pub fn new(goal: AiGoal) -> Self {
        Target {
            entity: None,
            priority: 1,
            goal: goal,
        }
    }
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

use std::fmt;

impl fmt::Display for Targets {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        for target in self.targets.iter() {
            write!(f, "{:?}\n", target)?;
        }

        Ok(())
    }
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

    /// DO NOT CALL DIRECTLY, since the AI would not know to create a new plan for the new target.
    /// Use finish_target instead.
    pub fn pop(&mut self) -> Option<Target> {
        self.targets.pop()
    }

    /// DO NOT CALL DIRECTLY. Use add_target instead.
    pub fn push(&mut self, target: Target) {
        self.targets.push(target);
    }
}
