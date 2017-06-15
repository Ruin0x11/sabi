use std::collections::HashSet;
use std::fmt;

use serde::de::{self, Deserialize, Deserializer, Visitor};
use serde::ser::{Serialize, Serializer};

use slog::Logger;

use item::{ItemContainer, ItemEffect};
use log;
use point::Point;
use stats::properties::Properties;

// For persistence between worlds, because the entity ID may change.
pub struct Uuid {

}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Name {
    pub name: String,
}

impl Name {
    pub fn new(name: &str) -> Self {
        Name {
            name: name.to_string()
        }
    }
}

#[cfg(never)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Position {
    x: i32,
    y: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Health {
    pub hit_points: i32,
    pub max_hit_points: i32,
}

impl Health {
    pub fn new(max: i32) -> Self {
        assert!(max > 0);
        Health {
            hit_points: max,
            max_hit_points: max,
        }
    }

    pub fn hurt(&mut self, amount: u32) {
        self.hit_points -= amount as i32;
    }

    pub fn kill(&mut self) {
        self.hit_points = 0;
    }

    pub fn is_dead(&self) -> bool {
        self.hit_points <= 0
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Appearance {
    pub kind: String
}

impl Appearance {
    pub fn new(kind: &str) -> Self {
        Appearance {
            kind: kind.to_string()
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Props {
    pub props: Properties,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Turn {
    pub speed: u32
}

impl Turn {
    pub fn new(speed: u32) -> Self {
        Turn {
            speed: speed,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Item {
    pub count: u32,
    pub weight: f32,
    pub effect: ItemEffect,
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Fov {
    pub visible: HashSet<Point>
}

impl Fov {
    pub fn new() -> Self {
        Fov {
            visible: HashSet::new(),
        }
    }

    pub fn is_visible(&self, pos: &Point) -> bool {
        self.visible.contains(pos)
    }
}

#[derive(Clone, Debug)]
pub struct Log {
    pub ident: String,
    pub logger: Logger,
}

lazy_static! {
    static ref MOB_LOG: Logger = log::make_logger("mob");
}

fn get_mob_log() -> Logger {
    MOB_LOG.new(o!())
}

impl Log {
    pub fn new(ident: &str) -> Self {
        Log {
            ident: ident.to_string(),
            logger: get_mob_log(),
        }
    }
}

impl Serialize for Log {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        serializer.serialize_str(&self.ident)
    }
}

impl<'de> Deserialize<'de> for Log {
    fn deserialize<D>(deserializer: D) -> Result<Log, D::Error>
        where D: Deserializer<'de>
    {
        struct LogVisitor;

        impl<'de> Visitor<'de> for LogVisitor {
            type Value = Log;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("`ident`")
            }

            fn visit_str<E>(self, value: &str) -> Result<Log, E>
                where E: de::Error
            {
                let logger = get_mob_log();
                let log = Log {
                    ident: value.to_string(),
                    logger: logger,
                };
                Ok(log)
            }
        }

        deserializer.deserialize_str(LogVisitor)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Inventory {
    pub container: ItemContainer,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Npc {

}

impl Npc {
    pub fn new() -> Self {
        Npc { }
    }
}
