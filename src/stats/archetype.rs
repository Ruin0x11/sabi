use std::collections::BTreeMap;

use toml::Value;

use graphics::Glyph;
use stats::*;
use stats::properties::*;
use util::toml::*;

const STATS_TABLE: &'static str = "stats";
const PROPERTIES_TABLE: &'static str = "properties";

pub struct Archetype {
    pub stats: Stats,
    pub properties: Properties,
    pub glyph: Glyph,
}

pub fn load(name: &str) -> Archetype {
    let value = toml_value_from_file(&format!("./data/monster/{}.toml", name));
    let archetype = make_archetype(value);
    archetype
}

fn make_archetype(value: Value) -> Archetype {
    let stats = make_stats(&value);
    let glyph = make_glyph(&value);
    let props = make_properties(&value);

    Archetype {
        stats: stats,
        glyph: glyph,
        properties: props,
    }
}

fn make_stats(value: &Value) -> Stats {
    // TEMP: Specify what stats are required based on the thing being instantiated.
    let init = StatsInit {
        hp:        expect_toml_value(value, STATS_TABLE, "hp"),
        strength:  expect_toml_value(value, STATS_TABLE, "strength"),
        defense:   expect_toml_value(value, STATS_TABLE, "defense"),
    };
    Stats::new(init)
}

fn make_glyph(value: &Value) -> Glyph {
    let glyph_str: String = expect_toml_value(value, STATS_TABLE, "glyph");

    match glyph_str.parse::<Glyph>() {
        Ok(g) => g,
        Err(..) => panic!("Glyph {} not found.", glyph_str),
    }
}

fn make_properties(value: &Value) -> Properties {
    match get_value_in_table(value, PROPERTIES_TABLE) {
        Some(&Value::Table(ref t)) => properties_from_table(t),
        Some(_)                    => panic!("[properties] was not a table!"),
        _                          => Properties::new(),
    }
}

fn properties_from_table(props_table: &BTreeMap<String, Value>) -> Properties {
    let mut props = Properties::new();
    for (key, val) in props_table.iter() {
        let prop_name = match key.parse::<Prop>() {
            Ok(name) => name,
            Err(..)  => panic!("No such property {} in the game.", key),
        };
        match *val {
            Value::Integer(i) => props.set(prop_name, i).unwrap(),
            Value::Boolean(b) => props.set(prop_name, b).unwrap(),
            _                 => panic!("Type {:?} isn't supported as a property.", val)
        };
    }
    props
}

#[cfg(test)]
mod tests {
    use super::*;
    use stats::properties::Prop::*;

    fn test_archetype(s: &str) -> Archetype {
        let value = toml_value_from_string(s);
        make_archetype(value)
    }

    #[test]
    #[should_panic]
    fn test_missing_required() {
        test_archetype("
[stats]
hp=10
strength=0
glyph=\"Prinny\"
");
    }

    #[test]
    #[should_panic]
    fn test_invalid_required() {
        test_archetype("
[stats]
hp=-10
strength=0
defense=0
glyph=\"Prinny\"
");
    }


    #[test]
    #[should_panic]
    fn test_invalid_prop() {
        test_archetype("
[stats]
hp=10
strength=10
defense=10
glyph=\"Prinny\"

[properties]
InvalidProp=true
");
    }

    #[test]
    #[should_panic]
    fn test_invalid_prop_table() {
        test_archetype("
[stats]
hp=10
strength=10
defense=10
glyph=\"Prinny\"

[[properties]]
");
    }

    #[test]
    fn test_instantiate_actor() {
        let arch = test_archetype("
[stats]
hp=20
strength=16
defense=18
glyph=\"Prinny\"

[properties]
TestNum=10
TestBool=true
");
        assert_eq!(arch.properties.get::<i64>(TestNum).unwrap(),   10);
        assert_eq!(arch.properties.get::<bool>(TestBool).unwrap(), true);
        assert_eq!(arch.glyph, Glyph::Prinny);
        assert_eq!(arch.stats.max_hp(), 20);
        assert_eq!(arch.stats.max_strength(), 16);
        assert_eq!(arch.stats.max_defense(), 18);
    }
}
