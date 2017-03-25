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
    use rand::{self, Rng};
    use super::*;
    use tile::{self, FLOOR};
    use glyph::Glyph;

    fn get_world() -> World {
        let mut world = World::generate(WorldType::Instanced(Point::new(32, 32)), 16, tile::WALL);
        world.draw_square(WorldPosition::new(15, 15), 10, tile::FLOOR);
        world
    }

    #[test]
    fn test_chunked_world() {
        let mut world = get_world();

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
        world.draw_square(Point::new(15, 15), 10, tile::FLOOR);
        start_with_params(player, world);
    }

    #[test]
    fn test_many_actors() {
        let mut rng = rand::thread_rng();
        let mut world = get_world();

        let mut player = Actor::new(6, 6, Glyph::Player);

        let values: [u32; 5] = [10, 50, 100, 200, 400];

        world.draw_square(Point::new(15, 15), 10, tile::FLOOR);

        for i in 1..10 {
            let mut other = Actor::new(10 + i, 10, Glyph::Player);
            other.speed = *rng.choose(&values).unwrap();
            world.add_actor(other);
        }

        start_with_params(player, world);
    }
}
