use point::Point;
use renderer::render::tilemap::{self, TileMap};
use GameContext;
use world::EcsWorld;

pub trait RenderUpdate {
    fn should_update(&self, context: &GameContext) -> bool;
    fn update(&mut self, context: &GameContext);
}
