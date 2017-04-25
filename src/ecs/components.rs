use glyph::Glyph;
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

#[cfg(never)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Inventory {
    pub containing: ItemContainer,
}
