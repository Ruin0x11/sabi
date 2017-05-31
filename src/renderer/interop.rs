use point::Point;
use glium::backend::Facade;
use renderer::render::tilemap::{self, TileMap};
use GameContext;
use world::EcsWorld;
use renderer::render::Viewport;

pub trait RenderUpdate {
    fn should_update(&self, context: &GameContext) -> bool;
    fn update(&mut self, context: &GameContext, viewport: &Viewport);

    fn redraw<F: Facade>(&mut self, display: &F, msecs: u64) {}
}
