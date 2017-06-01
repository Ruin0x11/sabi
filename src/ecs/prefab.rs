use ai::Ai;
use ecs::Loadout;
use ecs::components::*;

pub struct Prefab {
    pub loadout: Loadout,
}

pub fn mob(name: &str, health: i32, sprite: &str) -> Prefab {
    Prefab {
        loadout: Loadout::new()
            .c(Name::new(name))
            .c(Health::new(health))
            .c(Appearance::new(sprite))
            .c(Turn::new(100))
            .c(Ai::new())
            .c(Log::new("mob"))
    }
}
