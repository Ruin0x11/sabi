use actor::*;
use testbed::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integrate_goap() {
        let mut world = get_world();

        let mut player = Actor::from_archetype(10, 30, "test_player");
        player.disposition = Disposition::Friendly;
        player.name = String::from("You");

        for i in 0..48 {
            let other = Actor::from_archetype(10 + i, 32, "putit");
            world.add_actor(other);
        }

        start_with_params(player, world);
    }

    #[test]
    fn test_goap_step_once() {
        let mut world = get_world();

        let mut player = Actor::from_archetype(10, 30, "test_player");
        player.disposition = Disposition::Friendly;
        player.name = String::from("You");

        for i in 0..48 {
            let other = Actor::from_archetype(10 + i, 32, "putit");
            world.add_actor(other);
        }

        step_once(player, world);
    }
}
