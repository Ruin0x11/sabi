pub mod components;
pub mod instantiate;
pub mod prefab;
pub mod traits;

use ai;

Ecs! {
    healths: components::Health,
    names: components::Name,
    appearances: components::Appearance,
    turns: components::Turn,
    props: components::Props,
    ais: ai::Ai,
    logs: components::Log,
}
