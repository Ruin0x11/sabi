use ai::Ai;
use ecs::Loadout;
use ecs::components::*;

pub fn mob(name: &str, health: i32, sprite: &str) -> Loadout {
        Loadout::new()
            .c(Name::new(name))
            .c(Health::new(health))
            .c(Appearance::new(sprite))
            .c(Turn::new(100))
            .c(Ai::new())
            .c(Fov::new())
            .c(Log::new("mob"))
}

pub fn item(name: &str, sprite: &str) -> Loadout {
        Loadout::new()
            .c(Name::new(name))
            .c(Item::new())
            .c(Appearance::new(sprite))
            .c(Log::new("item"))
}
