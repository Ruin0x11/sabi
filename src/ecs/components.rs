use std::collections::HashSet;
use std::fmt;

use slog::Logger;

use data::Properties;
use item::{ItemContainer, ItemEffect};
use log;
use point::Point;
use util::clamp;
use macros::{Getter, Get};
use toml;
use uuid;

// For persistence between worlds, because the entity ID may change.
make_getter!(Uuid {
    uuid: uuid::Uuid
});

impl Default for Uuid {
    fn default() -> Self {
        Uuid::new()
    }
}

impl Uuid {
    pub fn new() -> Self {
        Uuid {
            uuid: uuid::Uuid::new_v4()
        }
    }
}

/// Interesting flags for mob entities.
make_getter!(Flags {
    is_invulnerable: bool,
    reflects_ranged: bool,
});

impl Default for Flags {
    fn default() -> Self {
        Flags {
            is_invulnerable: false,
            reflects_ranged: false,
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum Gender {
    Male,
    Female,
    Unknown,
}

make_getter!(Name {
    name: String,
    proper_name: Option<String>,
    gender: Gender,
});

impl Default for Name {
    fn default() -> Self {
        Name::new("someone".to_string())
    }
}

impl Name {
    pub fn new(name: String) -> Self {
        Name {
            name: name.to_string(),
            proper_name: None,
            gender: Gender::Unknown,
        }
    }

    pub fn new_proper(name: String, proper_name: String) -> Self {
        Name {
            name: name.to_string(),
            proper_name: Some(proper_name.to_string()),
            gender: Gender::Unknown,
        }
    }
}


make_getter!(Props {
    props: Properties
});

impl Default for Props {
    fn default() -> Self {
        Props::new()
    }
}

impl Props {
    pub fn new() -> Self {
        Props {
            props: Properties::new(),
        }
    }
}


make_getter!(Health {
    hit_points: i32,
    max_hit_points: i32,
    tp: i32,
    max_tp: i32,
});

impl Default for Health {
    fn default() -> Self {
        Health::new(1)
    }
}

impl Health {
    pub fn new(max: i32) -> Self {
        assert!(max > 0);
        Health {
            hit_points: max,
            max_hit_points: max,
            tp: 0,
            max_tp: 100,
        }
    }

    pub fn percent(&self) -> f32 {
        self.hit_points as f32 / self.max_hit_points as f32
    }

    pub fn hurt(&mut self, amount: u32) {
        self.hit_points -= amount as i32;
    }

    pub fn heal(&mut self, amount: u32) {
        self.hit_points += amount as i32;
        if self.hit_points >= self.max_hit_points {
            self.hit_points = self.max_hit_points;
        }
    }

    pub fn tp_full(&self) -> bool {
        self.tp == self.max_tp
    }

    pub fn adjust_tp(&mut self, amount: i32) {
        self.tp = clamp(self.tp + amount, 0, self.max_tp);
    }

    pub fn reset_tp(&mut self) {
        self.tp = 0;
    }

    pub fn kill(&mut self) {
        self.hit_points = 0;
    }

    pub fn is_dead(&self) -> bool {
        self.hit_points <= 0
    }
}


make_getter!(Appearance {
    kind: String,
});

impl Default for Appearance {
    fn default() -> Self {
        Appearance::new("unknown")
    }
}

impl Appearance {
    pub fn new(kind: &str) -> Self {
        Appearance { kind: kind.to_string() }
    }
}


make_getter!(Turn {
    speed: u32,
});

impl Default for Turn {
    fn default() -> Self {
        Turn::new(100)
    }
}

impl Turn {
    pub fn new(speed: u32) -> Self {
        Turn { speed: speed }
    }
}


make_getter!(Item {
    count: u32,
    weight: f32,
    effect: ItemEffect,
});

impl Default for Item {
    fn default() -> Self {
        Item::new()
    }
}

impl Item {
    pub fn new() -> Self {
        Item {
            count: 1,
            weight: 0.0,
            effect: ItemEffect::Heal,
        }
    }

    pub fn weight(&self) -> f32 {
        self.count as f32 * self.weight
    }

    pub fn can_merge(&self, other: &Item) -> bool {
        false
    }

    pub fn merge(&mut self, other: &Item) {
        self.count += other.count;
    }
}

make_getter!(Fov {
    visible: HashSet<Point>,
});

impl Default for Fov {
    fn default() -> Self {
        Fov::new()
    }
}

impl Fov {
    pub fn new() -> Self {
        Fov { visible: HashSet::new() }
    }

    pub fn is_visible(&self, pos: &Point) -> bool {
        self.visible.contains(pos)
    }
}


make_getter!(Log {
    ident: String,
});

lazy_static! {
    static ref MOB_LOG: Logger = log::make_logger("mob");
}

fn get_mob_log() -> Logger {
    MOB_LOG.new(o!())
}

impl Default for Log {
    fn default() -> Self {
        Log::new("mob")
    }
}

impl Log {
    pub fn new(ident: &str) -> Self {
        Log {
            ident: ident.to_string(),
        }
    }

    pub fn get(&self) -> &'static Logger {
        &MOB_LOG
    }
}

make_getter!(Inventory {
    container: ItemContainer,
});


impl Default for Inventory {
    fn default() -> Self {
        Inventory {
            container: ItemContainer::new(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Slot {
    pub kind: SlotKind,
    pub name: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SlotKind {
    Arm,
    Head,
    Legs
}

impl fmt::Display for Slot {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let kind_str = match self.kind {
            SlotKind::Arm => "arm",
            SlotKind::Head => "head",
            SlotKind::Legs => "legs",
        };
        write!(f, "{}", kind_str)
    }
}


make_getter!(Equipment {
    slots: Vec<Slot>,
});

impl Default for Equipment {
    fn default() -> Self {
        Equipment::new(Vec::new())
    }
}

impl Equipment {
    pub fn new(slots: Vec<Slot>) -> Self {
        Equipment {
            slots: slots,
        }
    }

    pub fn can_equip(&self, slot_idx: usize) -> bool {
        let slot = match self.slots.get(slot_idx) {
            Some(s) => s,
            None    => return false,
        };

        true
    }
}


make_getter!(Npc {
    quests: Vec<String>,
});

impl Default for Npc {
    fn default() -> Self {
        Npc::new()
    }
}

impl Npc {
    pub fn new() -> Self {
        Npc {
            quests: Vec::new()
        }
    }
}
