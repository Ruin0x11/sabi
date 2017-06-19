use ai::{Ai, AiKind};
use ecs::Loadout;
use ecs::components::*;
use data::namegen;

pub fn mob(name: &str, health: i32, sprite: &str) -> Loadout {
    Loadout::new()
        .c(Name::new_proper(name.to_string(), namegen::gen()))
        .c(Health::new(health))
        .c(Appearance::new(sprite))
        .c(Turn::new(100))
        .c(Ai::new(AiKind::SeekTarget))
        .c(Fov::new())
        .c(Log::new("mob"))
}

pub fn npc(name: &str) -> Loadout {
    mob(name, 1000, "npc")
        .c(Npc::new())
        .c(Ai::new(AiKind::Wait))
}

pub fn item(name: &str, sprite: &str) -> Loadout {
    Loadout::new()
        .c(Name::new(name.to_string()))
        .c(Item::new())
        .c(Appearance::new(sprite))
        .c(Log::new("item"))
}
