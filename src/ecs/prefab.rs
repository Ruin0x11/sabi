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

fn mob_unnamed(name: &str, health: i32, sprite: &str) -> Loadout {
    Loadout::new()
        .c(Name::new(name.to_string()))
        .c(Health::new(health))
        .c(Appearance::new(sprite))
        .c(Turn::new(100))
        .c(Ai::new(AiKind::SeekTarget))
        .c(Fov::new())
        .c(Log::new("mob"))
}

static MOBS_COMMON: [(&str, &str); 12] = [
    ("wolf", "wolf"),
    ("dog", "dog"),
    ("fuzzball", "fuzzball"),
    ("owl", "owl"),
    ("duck", "duck"),
    ("pigeon", "pigeon"),
    ("snow_bird", "snow bird"),
    ("cow", "cow"),
    ("pig", "pig"),
    ("putit", "putit"),
    ("prawn", "prawn"),
    ("yeek", "yeek"),
];

static MOBS_RARE: [(&str, &str); 8] = [
    ("blade", "blade"),
    ("bike", "bike"),
    ("sandwich", "living sandwich"),
    ("candle", "candle"),
    ("thing", "thing"),
    ("street_sweeper", "street sweeper"),
    ("hair_woman", "hair woman"),
    ("cockatrice", "cockatrice"),
];

static MOBS_HUMANOID: [(&str, &str); 8] = [
    ("mage", "mage"),
    ("hitman", "hitman"),
    ("miko", "miko"),
    ("maid", "maid"),
    ("archer", "archer"),
    ("mage2", "oriental mage"),
    ("markswoman", "markswoman"),
    ("lawyer", "lawyer"),
];

pub fn random_mob() -> Loadout {
    let mut rng = rand::thread_rng();
    let mut named = false;
    let &(sprite, name) = if rand_util::chance(0.1, &mut rng) {
        if rand_util::chance(0.3, &mut rng) {
            rng.choose(&MOBS_RARE).unwrap()
        } else {
            named = true;
            rng.choose(&MOBS_HUMANOID).unwrap()
        }
    } else {
        rng.choose(&MOBS_COMMON).unwrap()
    };

    if named {
        mob(name, 250, sprite)
    } else {
        mob_unnamed(name, 30, sprite)
    }
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

static ITEMS_RARE: [(&str, &str); 7] = [
    ("bass", "bass guitar"),
    ("violin", "violin"),
    ("shield3", "large shield"),
    ("glove3", "pair of decent gloves"),
    ("winchester", "Winchester premium"),
    ("amazon", "amazon.co.jp delivery"),
    ("sword", "nice sword"),
];

pub fn random_item() -> Loadout {
    let mut rng = rand::thread_rng();
    let &(sprite, name) = if rand_util::chance(0.1, &mut rng) {
        rng.choose(&ITEMS_RARE).unwrap()
    } else {
        rng.choose(&ITEMS_COMMON).unwrap()
    };

    item(name, sprite)
}
