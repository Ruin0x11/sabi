mod chunk_management;
mod components;
pub mod prefab;
pub mod traits;
mod spatial;
mod world;
mod flags;

pub use self::world::EcsWorld;
pub use self::traits::*;

Ecs! {
    healths: components::Health,
    names: components::Name,
    appearances: components::Appearance,
    turns: components::Turn,
    props: components::Props,
}
