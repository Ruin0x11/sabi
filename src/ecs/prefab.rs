use ecs::Loadout;
use ecs::components::*;
use glyph::Glyph;

pub struct Prefab {
    pub loadout: Loadout,
}

pub fn mob(name: &str, glyph: Glyph) -> Prefab {
    Prefab {
        loadout: Loadout::new()
            .c(Name::new(name))
            .c(Health::new(100))
            .c(Appearance::new(glyph))
            .c(Turn::new(100))
    }
}
