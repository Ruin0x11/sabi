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
    {
        let context_mut = &mut context;

        while !context_mut.canvas.window_closed() {
            state::process(context_mut);
        }
    }
    info!(context.logger, "Testbed exited cleanly.");
}

/// Creates an object of the specified type from a grid string using constructor
/// and callback closures. Typically used for quickly producing 2D grids/worlds.
/// The grid string may look something like this:
/// ```
/// .....
/// .#.#.
/// ..@..
/// .#.#.
/// .....
/// ```
pub fn make_grid_from_str<M, F, T>(text: &str, mut constructor: M, mut callback: F) -> T
    where M: FnMut(Point) -> T,
          F: FnMut(&Point, char, &mut T) {
    let mut x = 0;
    let mut y = 0;

    let lines = text.split('\n').filter(|l| l.len() > 0).collect::<Vec<_>>();
    let height = lines.len();
    assert!(height > 0);
    let width = lines[0].len();
    assert!(width > 0);
    assert!(lines.iter().all(|line| line.chars().count() == width));
    let mut thing = constructor(Point::new(width as i32, height as i32));

    for line in lines {
        for ch_at_point in line.chars() {
            let grid_pos = Point { x: x as i32, y: y as i32 };
            callback(&grid_pos, ch_at_point, &mut thing);

            x += 1;
        }
        y += 1;
        x = 0;
    }

    thing
}


#[cfg(test)]
    mod tests {
    use rand::{self};
    use rand::distributions::{IndependentSample, Range};
    use super::*;
    use tile;
    use glyph::Glyph;

    fn get_world() -> World {
        let mut world = World::generate(WorldType::Instanced(WorldPosition::new(32, 32)),
                                        16, tile::WALL);
        world.draw_square(WorldPosition::new(15, 15), 10, tile::FLOOR);
        world
    }

    #[test]
    fn test_chunked_world() {
        let world = get_world();

        let player = Actor::new(0, 0, Glyph::Player);
        start_with_params(player, world);
    }


    #[test]
    fn test_no_actors() {
        let mut world = get_world();

        let mut player = Actor::new(6, 6, Glyph::Player);
        player.speed = 300;

        world.draw_square(Point::new(15, 15), 10, tile::FLOOR);
        start_with_params(player, world);
    }

    #[test]
    fn test_one_actor() {
        let mut world = get_world();

        let mut player = Actor::new(6, 6, Glyph::Player);
        player.speed = 300;

        let mut other = Actor::from_archetype(10, 10, "prinny");
        other.speed = 100;
        world.add_actor(other);
        world.draw_square(Point::new(15, 15), 10, tile::FLOOR);
        start_with_params(player, world);
    }

    #[test]
    fn test_many_actors() {
        let mut rng = rand::thread_rng();
        let mut world = get_world();

        let player = Actor::new(6, 6, Glyph::Player);

        world.draw_square(Point::new(15, 15), 10, tile::FLOOR);
        let range = Range::new(30, 200);

        for i in 0..8 {
            let mut other = Actor::from_archetype(10 + i, 10, "prinny");
            other.speed = range.ind_sample(&mut rng);
            world.add_actor(other);
        }

        for i in 0..8 {
            let mut other = Actor::from_archetype(10 + i, 11, "putit");
            other.speed = range.ind_sample(&mut rng);
            world.add_actor(other);
        }

        start_with_params(player, world);
    }
}
