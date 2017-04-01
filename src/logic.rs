use world::World;
use action::Action;
use actor::{Actor, ActorId};
use event::*;

fn pre_tick(world: &mut World) {

}

fn pre_tick_actor(world: &mut World, actor: &Actor) {

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
    let delay = (100*100 / actor.speed) as i32;
    let name = if actor.is_player(world) {
        "[Player]"
    } else {
        "actor"
    };
    debug!(actor.logger, "{}: delay {}, speed {}", name, delay, actor.speed);
    world.add_delay_for(&actor.get_id(), delay);
    actor.update_fov(world);
}

fn post_tick(world: &mut World) {

}

fn run_actor_action(world: &mut World, actor: &mut Actor, action: Action) {
    match action {
        Action::Move(dir) => actor.move_in_direction(dir, world),
        Action::Dood => world.message("Dood!".to_string()),
        Action::Wait => (),
        Action::Explod => {
            let pos = actor.get_pos();
            world.message("Ex!".to_string());
            world.events.push(Event {
                area: EventArea::Square(pos.x, pos.y, 5),
                kind: EventKind::SayName,
            })
        },
        Action::Hurt => world.message("Oof!".to_string())
    }
}
