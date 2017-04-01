use world::World;
use action::Action;
use actor::{Actor, Direction, ActorId};
use event::*;
use stats;

fn pre_tick(_world: &mut World) {

}

fn pre_tick_actor(_world: &mut World, _actor: &Actor) {

}

pub fn run_action(world: &mut World, id: &ActorId, action: Action) {
    pre_tick(world);
    world.with_actor(id, |mut world, mut actor| {
        debug!(actor.logger, "Action: {:?}", action);

        pre_tick_actor(world, &actor);
        run_actor_action(world, &mut actor, action.clone());
        post_tick_actor(world, &actor);
    });

    post_tick(world);
}

fn post_tick_actor(world: &mut World, actor: &Actor) {
    // TEMP: speed algorithm is needed.
    let delay = stats::formulas::calculate_delay(actor, 100);
    let name = if actor.is_player(world) {
        "[Player]"
    } else {
        "actor"
    };
    debug!(actor.logger, "{}: delay {}, speed {}", name, delay, actor.speed);
    world.add_delay_for(&actor.get_id(), delay);
    actor.update_fov(world);
}

fn post_tick(_world: &mut World) {

}

fn try_move(world: &mut World, actor: &mut Actor, dir: Direction) {
    let new_pos = Direction::add_offset(actor.get_pos(), dir);
    let id_opt = world.actor_id_at(new_pos);
    if let Some(id) = id_opt {
        swing_at(world, actor, id);
    } else {
        actor.move_to(new_pos, world);
    }
}

fn swing_at(world: &mut World, attacker: &mut Actor, other_id: ActorId) {
    let damage;
    let evaded;
    {
        let other = world.actor(&other_id);
        assert!(attacker.get_pos().next_to(other.get_pos()), "Tried swinging from out of range! (could be implemented)");
        evaded = stats::formulas::check_evasion(attacker, other);
        if evaded {
            world.message("Evaded!".to_string());
            return;
        }
        damage = stats::formulas::calculate_damage(attacker, other);
    }
    world.with_actor(&other_id, |world, other| {
        world.message(format!("{} hits {}! {} damage!", attacker, other, damage));
        other.hurt(damage)
    })
}

fn run_actor_action(world: &mut World, actor: &mut Actor, action: Action) {
    match action {
        Action::Move(dir) => try_move(world, actor, dir),
        Action::Dood => world.message(format!("{}: \"Dood!\"", actor.get_display_name())),
        Action::Wait => (),
        Action::Explod => {
            let pos = actor.get_pos();
            world.message(format!("{} explodes!", actor.get_display_name()));
            world.events.push(Event {
                area: EventArea::Square(pos.x, pos.y, 5),
                kind: EventKind::SayName,
            })
        },
        Action::Hurt => world.message(format!("{}: \"Oof\"!", actor.get_display_name()))
    }
}
