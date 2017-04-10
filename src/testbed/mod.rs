mod goap_test;
pub mod item;
pub mod items;

use std::cell::RefCell;

use actor::*;
use world::*;
use state;
use ::*;
use self::item::*;
use glyph::Glyph;

lazy_static! {
    static ref DESC: ItemDesc = ItemDesc { name: "dream",
                                           id: 1,
                                           weight: 0.1,
                                           container: false,
                                           glyph: Glyph::Item };
}

pub fn get_world<'a>() -> World {
    let mut world = World::generate(WorldType::Instanced(WorldPosition::new(64, 64)),
                                    16, tile::WALL);
    world.draw_square(WorldPosition::new(32, 32), 30, tile::FLOOR);
    let mut item = Item::new(&DESC);
    world.add_item(WorldPosition::new(5, 5), item);

    world
}

pub fn step_once(player: Actor, world: World) {
    init();
    let mut context = get_context();
    context.state.set_world(world);
    {
        let mut world = &mut context.state.world;
        world.set_player_id(player.get_id());
        world.add_actor(player);
    }
    state::step(&mut context);
}

pub fn start_with_params(player: Actor, world: World) {
    init();
    let mut context = get_context();
    context.state.set_world(world);
    {
        let mut world = &mut context.state.world;
        world.set_player_id(player.get_id());
        world.add_actor(player);
    }
    while !canvas::window_closed() {
        state::process(&mut context);
    }
    println!("Done.");
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

    #[test]
    fn test_chunked_world() {
        let world = get_world();

        let player = Actor::from_archetype(0, 0, "test_player");
        start_with_params(player, world);
    }

    #[test]
    fn test_no_actors() {
        let mut world = get_world();

        let mut player = Actor::from_archetype(6, 6, "test_player");
        player.speed = 300;

        world.draw_square(Point::new(15, 15), 10, tile::FLOOR);
        start_with_params(player, world);
    }

    #[test]
    fn test_one_actor() {
        let mut world = get_world();

        let mut player = Actor::from_archetype(6, 6, "test_player");
        player.speed = 300;
        player.disposition = Disposition::Friendly;

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

        let mut player = Actor::from_archetype(6, 6, "test_player");
        player.disposition = Disposition::Friendly;
        player.name = String::from("You");

        let range = Range::new(30, 200);

        for i in 0..32 {
            let mut other = Actor::from_archetype(10 + i, 48, "putit");
            other.speed = range.ind_sample(&mut rng);
            world.add_actor(other);
        }

        start_with_params(player, world);
    }
}
