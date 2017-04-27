use ai::Ai;
use ecs::Loadout;
use ecs::components::*;
use fov::FieldOfView;
use glyph::Glyph;

pub struct Prefab {
    pub loadout: Loadout,
}

impl Prefab {
    pub fn new() -> Prefab {
        mob("Mob", 10, Glyph::Putit)
    }
}

pub fn mob(name: &str, health: i32, glyph: Glyph) -> Prefab {
    Prefab {
        loadout: Loadout::new()
            .c(Name::new(name))
            .c(Health::new(health))
            .c(Appearance::new(glyph))
            .c(Turn::new(100))
            .c(FieldOfView::new())
            .c(Ai::new())
            .c(Log::new("mob"))
    }
}
