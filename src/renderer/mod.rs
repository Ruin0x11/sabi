mod atlas;
mod render;
mod util;
pub mod ui;

mod traits;

pub use self::render::{Action, RenderContext};
pub use self::traits::RenderUpdate;

make_global!(RENDERER, RenderContext, RenderContext::new());

pub use self::instance::*;
