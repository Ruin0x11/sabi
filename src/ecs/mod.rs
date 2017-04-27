pub mod components;
pub mod instantiate;
pub mod prefab;

use ai;
use fov;

Ecs! {
    healths: components::Health,
    names: components::Name,
    appearances: components::Appearance,
    turns: components::Turn,
    props: components::Props,
    ais: ai::Ai,
    fovs: fov::FieldOfView,
    logs: components::Log,
}
