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

pub fn make_from_str<M, F, T>(text: &str, mut make: M, mut callback: F) -> T
    where M: FnMut(Point) -> T,
          F: FnMut(&Point, char, &mut T) {
    let mut start = Point{x: 0, y: 0};
    let mut destination = Point{x: 0, y: 0};
    let mut x = 0;
    let mut y = 0;

    let lines = text.split('\n').filter(|l| l.len() > 0).collect::<Vec<_>>();
    let height = lines.len();
    assert!(height > 0);
    let width = lines[0].len();
    assert!(width > 0);
    assert!(lines.iter().all(|line| line.chars().count() == width));
    let mut thing = make(Point::new(height as i32, width as i32));

    for line in lines {
        for c in line.chars() {
            let pt = Point { x: x as i32, y: y as i32 };
            callback(&pt, c, &mut thing);

            x += 1;
        }
        y += 1;
        x = 0;
    }

    thing
}


#[cfg(test)]
    mod tests {
    use rand::{self, Rng};
    use super::*;
    use tile::{self, FLOOR};
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

        let mut other = Actor::new(10, 10, Glyph::Dood);
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

        let values: [u32; 4] = [10, 50, 100, 200];

        world.draw_square(Point::new(15, 15), 10, tile::FLOOR);

        for i in 1..16 {
            let mut other = Actor::new(10 + i, 10, Glyph::Dood);
            other.speed = *rng.choose(&values).unwrap();
            world.add_actor(other);
        }

        start_with_params(player, world);
    }
}
