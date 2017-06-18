use glium::backend::Facade;
use state::GameState;
use renderer::render::Viewport;

pub trait RenderUpdate {
    fn should_update(&self, state: &GameState) -> bool;
    fn update(&mut self, state: &GameState, viewport: &Viewport);

    fn redraw<F: Facade>(&mut self, _display: &F, _msecs: u64) {}
}
