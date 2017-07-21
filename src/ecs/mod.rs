pub mod components;
pub mod globals;
pub mod instantiate;
pub mod party;
pub mod prefab;
pub mod traits;

use self::components::*;
use util;
use ai::Ai;

use macros::{Getter, TomlInstantiate};
use toml;

#[derive(Eq, PartialEq, Debug)]
enum CompoType {
    Instantiable,
    Static,
}

use self::CompoType::*;

macro_rules! Ecs_with_toml {
    {
        $($compname:ident: $comptype:ty, $should_get:expr;)+
    } => {

        Ecs! {
            $($compname: $comptype,)*
        }

        pub fn load_mob(name: &str) -> Result<Loadout, ()> {
            loadout_from_toml_file(&format!("data/monster/{}.toml", name))
        }

        fn loadout_from_toml_file(filename: &str) -> Result<Loadout, ()> {
            let mut loadout = Loadout::new();
            let val = util::toml::toml_value_from_file(filename);

            if let Some(parent) = util::toml::get_toml_value::<String>(&val, "Meta", "parent") {
                loadout = load_mob(&parent).unwrap();
            }

            let components = match util::toml::value_in_table(&val, "Components") {
                Some(&toml::Value::Table(ref table)) => table,
                _ => return Err(()),
            };
            let names = components.keys().cloned();

            for name in names {
                let compo_table = match components[&name] {
                    toml::Value::Table(ref table) => table,
                    _ => return Err(()),
                };

                $(
                    if name == stringify!($comptype) {
                        let compo: $comptype = if $should_get == CompoType::Instantiable {
                            TomlInstantiate::get_for(compo_table)?
                        } else {
                            Default::default()
                        };
                        loadout = loadout.c(compo);
                    }
                )*
            }

            Ok(loadout)
        }
    }
}

Ecs_with_toml! {
    healths: Health, Instantiable;
    names: Name, Instantiable;
    appearances: Appearance, Instantiable;
    turns: Turn, Instantiable;
    items: Item, Instantiable;
    invs: Inventory, Instantiable;
    ais: Ai, Instantiable;
    fovs: Fov, Instantiable;
    npcs: Npc, Instantiable;

    uuids: Uuid, Static;
    props: Props, Static;
    logs: Log, Static;
}
