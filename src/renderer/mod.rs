mod atlas;
mod render;
mod util;
mod ui;

mod interop;

pub use self::render::{Action, RenderContext};
pub use self::interop::RenderUpdate;

make_global!(RENDERER, RenderContext, RenderContext::new());

pub use self::instance::*;
