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
    items: components::Item,
    invs: components::Inventory,
    ais: ai::Ai,
    fovs: components::Fov,
    logs: components::Log,
}
