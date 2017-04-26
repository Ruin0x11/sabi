use std::fmt;

use serde::de::{self, Deserialize, Deserializer, Visitor};
use serde::ser::{Serialize, Serializer, SerializeStruct};

use slog::Logger;

use glyph::Glyph;
use log;
use stats::properties::Properties;

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

    pub fn is_dead(&self) -> bool {
        self.hit_points <= 0
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Appearance {
    pub glyph: Glyph,
}

impl Appearance {
    pub fn new(glyph: Glyph) -> Self {
        Appearance {
            glyph: glyph,
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
    pub can_equip: bool,
    pub count: u32,
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

impl Deserialize for Log {
    fn deserialize<D>(deserializer: D) -> Result<Log, D::Error>
        where D: Deserializer
    {
        struct LogVisitor;

        impl Visitor for LogVisitor {
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

#[cfg(never)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Inventory {
    pub containing: ItemContainer,
}
