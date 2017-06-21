pub mod components;
pub mod globals;
pub mod instantiate;
pub mod prefab;
pub mod traits;

use self::components::*;
use util;
use ai::Ai;

use macros::{Getter, Get};
use toml;

macro_rules! Ecs_with_toml {
    {
        $($compname:ident: $comptype:ty, $should_get:expr;)+
    } => {

        Ecs! {
            $($compname: $comptype,)*
        }

        pub fn loadout_from_toml_file(filename: &str) -> Result<Loadout, ()> {
            let mut loadout = Loadout::new();
            let val = util::toml::toml_value_from_file(filename);
            let table = match val {
                toml::Value::Table(ref table) => table,
                _ => return Err(()),
            };

            let names = table.keys().cloned();

            for name in names {
                $(
                    let compo_table = match table[&name] {
                        toml::Value::Table(ref table) => table,
                        _ => return Err(()),
                    };

                    if name == stringify!($comptype) {
                        let compo: $comptype = if $should_get {
                            Get::get_for(compo_table)?
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
    healths: Health, true;
    names: Name, true;
    appearances: Appearance, true;
    turns: Turn, true;
    props: Props, false;
    items: Item, true;
    invs: Inventory, true;
    ais: Ai, true;
    fovs: Fov, true;
    npcs: Npc, true;
    logs: Log, false;
}
