use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use serde::Deserialize;
use toml::Value;

use glyph::Glyph;
use stats::*;
use stats::properties::*;

const STATS_TABLE: &'static str = "stats";
const PROPERTIES_TABLE: &'static str = "properties";

pub struct Archetype {
    pub stats: Stats,
    pub properties: Properties,
    pub glyph: Glyph,
}

pub fn load(name: &str) -> Archetype {
    let data_str = parse_file(name);
    let archetype = make_archetype(data_str);
    archetype
}

fn parse_file(name: &str) -> String {
    let path = PathBuf::from(format!("./data/{}.toml", name));
    let mut file = File::open(&path).expect("No open file!");
    let mut data = String::new();
    file.read_to_string(&mut data).expect("Can't read file!");
    data
}

fn get_toml_value<T: Deserialize>(value: &Value, table: &str, key: &str) -> T {
    match value[table][key].clone().try_into::<T>() {
        Ok(v) => v,
        Err(..) => panic!("Required value \"{}.{}\" not found!", table, key),
    }
}

fn make_archetype(data_str: String) -> Archetype {
    let value = data_str.parse::<Value>().expect("Invalid TOML!");

    // TEMP: Specify what fields are required based on the thing being instantiated.
    let stats = StatsInit {
        hp:        get_toml_value(&value, STATS_TABLE, "hp"),
        strength:  get_toml_value(&value, STATS_TABLE, "strength"),
        defense:   get_toml_value(&value, STATS_TABLE, "defense"),
    };

    let glyph_name: String = get_toml_value(&value, STATS_TABLE, "glyph");

    let glyph = match glyph_name.parse::<Glyph>() {
        Ok(g) => g,
        Err(..) => panic!("Glyph {} not found.", glyph_name),
    };

    let props_table = match value[PROPERTIES_TABLE] {
        Value::Table(ref t) => t,
        _               => panic!("[properties] was not a table!"),
    };

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

    Archetype {
        stats: Stats::new(stats),
        glyph: glyph,
        properties: props,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use stats::properties::Prop::*;

    fn test_archetype(s: &str) -> Archetype {
        make_archetype(s.to_string())
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
properties=42
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
