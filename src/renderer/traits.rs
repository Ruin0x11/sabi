use glium::backend::Facade;
use world::World;
use renderer::render::Viewport;

pub trait RenderUpdate {
    fn should_update(&self, world: &World) -> bool;
    fn update(&mut self, world: &World, viewport: &Viewport);

    fn redraw<F: Facade>(&mut self, _display: &F, _msecs: u64) {}
}
