use actor::*;
use glyph::Glyph;
use world::*;
use state;
use ::*;
 
pub fn start_with_params(player: Actor, world: World) {
    init();
    let mut context = get_context();
    context.state.set_world(world);
    context.state.current_world_mut().set_player_id(player.get_uuid());
    context.state.current_world_mut().add_actor(player);

    let context_mut = &mut context;

    while !context_mut.canvas.window_closed() {
        state::process(context_mut);
    }
}

#[cfg(test)]
mod tests {
    use tile::*;
    use glyph::*;
    use super::*;

    fn get_world() -> World {
        World::generate(WorldType::Instanced(Point::new(32, 32)), 16)
    }

    #[test]
    fn test_chunked_world() {
        let mut world = World::generate(WorldType::Instanced(Point::new(32, 32)), 16);

        let tile = Tile {
            type_: TileType::Wall,
            glyph: Glyph::Wall,
            feature: None,
        };

        world.draw_rect(WorldPosition::new(1, 1),
                        WorldPosition::new(30, 30),
                        tile);

        let player = Actor::new(0, 0, Glyph::Player);
        start_with_params(player, world);
    }

    #[test]
    fn test_one_actor() {
        let mut world = get_world();

        let player = Actor::new(0, 0, Glyph::Player);

        let other = Actor::new(10, 10, Glyph::Player);
        world.add_actor(other);
        start_with_params(player, world);
    }
}
