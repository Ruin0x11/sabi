use actor::*;
use world::*;
use state;
use ::*;

pub fn start_with_params(player: Actor, world: World) {
    init();
    let mut context = get_context();
    context.state.set_world(world);
    context.state.current_world_mut().set_player_id(player.get_id());
    context.state.current_world_mut().add_actor(player);

    let context_mut = &mut context;

    while !context_mut.canvas.window_closed() {
        state::process(context_mut);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tile::*;
    use glyph::Glyph;

    fn get_world() -> World {
        World::generate(WorldType::Instanced(Point::new(32, 32)), 16)
    }

    #[test]
    fn test_chunked_world() {
        let mut world = World::generate(WorldType::Instanced(Point::new(32, 32)), 16);

        let tile = Tile {
            type_: TileType::Floor,
            glyph: Glyph::Floor,
            feature: None,
        };

        world.draw_square(WorldPosition::new(15, 15), 10, tile);

        let player = Actor::new(0, 0, Glyph::Player);
        start_with_params(player, world);
    }

    #[test]
    fn test_one_actor() {
        let mut world = get_world();

        let mut player = Actor::new(6, 6, Glyph::Player);
        player.speed = 300;

        let other = Actor::new(10, 10, Glyph::Player);
        world.add_actor(other);
        world.draw_square(Point::new(15, 15), 10, Tile {
            type_: TileType::Floor,
            glyph: Glyph::Floor,
            feature: None,
        });
        start_with_params(player, world);
    }
}
