use glium::backend::Facade;
use GameContext;
use renderer::render::Viewport;

pub trait RenderUpdate {
    fn should_update(&self, context: &GameContext) -> bool;
    fn update(&mut self, context: &GameContext, viewport: &Viewport);

    fn redraw<F: Facade>(&mut self, _display: &F, _msecs: u64) {}
}
