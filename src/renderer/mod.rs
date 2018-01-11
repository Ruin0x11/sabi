mod atlas;
mod render;
mod util;
pub mod ui;

mod traits;

pub use self::render::{Action, RenderContext};
pub use self::traits::RenderUpdate;

make_global!(RENDERER, RenderContext, RenderContext::new());

pub use self::instance::*;

pub fn wait(frames: usize) {
    with_mut(|renderer| {
        let mut c = 0;
        while c < frames {
            renderer.render();
            renderer.step_frame();
            c += 1;
        }
    });
}
