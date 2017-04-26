pub mod components;
pub mod prefab;
pub mod traits;
pub mod flags;
pub mod instantiate;

use ai;
use fov;

pub use self::traits::*;

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
