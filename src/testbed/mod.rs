use actor::*;
use world::*;
use state;
use ::*;
 
pub fn start_with_params(player: Actor, world: World) {
    init();
    let mut context = get_context();
    let context_mut = &mut context;
    context_mut.state.set_world(world);
    context_mut.state.set_player(player);

    while !context_mut.canvas.window_closed() {
        state::process(context_mut);
    }
}

#[cfg(test)]
mod tests {
    use tile::*;
    use glyph::*;
    use super::*;

    #[test]
    pub fn test_chunked_world() {
        let mut world = World::generate(WorldType::Instanced(Point::new(32, 32)), 16);

        let tile = Tile {
            type_: TileType::Wall,
            glyph: Glyph::Wall,
            feature: None,
        };

        world.draw_rect(WorldPosition::new(1, 1),
                        WorldPosition::new(30, 30),
                        tile);

        let player = Actor::new(0, 0);
        start_with_params(player, world);
    }
}
