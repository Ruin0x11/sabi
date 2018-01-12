use ai::{Ai, AiKind};
use ecs::Loadout;
use ecs::components::*;
use data::namegen;
use util::rand_util;
use rand::{self, Rng};

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

static ITEMS_COMMON: [(&str, &str); 13] = [
    ("coffee", "coffee"),
    ("acorn", "acorn"),
    ("sunflower_seed", "sunflower seed"),
    ("strawberry", "strawberry"),
    ("grapes", "grapes"),
    ("leek", "leek"),
    ("tomato", "tomato"),
    ("watermelon", "watermelon"),
    ("panty", "pair of panties"),
    ("glove", "pair of gloves"),
    ("glove2", "pair of leather gloves"),
    ("shield", "small shield"),
    ("shield2", "shield"),
];

static ITEMS_RARE: [(&str, &str); 6] = [
    ("bass", "bass guitar"),
    ("violin", "violin"),
    ("shield3", "large shield"),
    ("glove3", "pair of decent gloves"),
    ("winchester", "Winchester premium"),
    ("amazon", "amazon.co.jp delivery"),
];

pub fn random_item() -> Loadout {
    let mut rng = rand::thread_rng();
    let &(sprite, name) = if rand_util::chance(0.15, &mut rng) {
        rng.choose(&ITEMS_RARE).unwrap()
    } else {
        rng.choose(&ITEMS_COMMON).unwrap()
    };

    item(name, "sword")
}
