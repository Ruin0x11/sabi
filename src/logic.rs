use calx_ecs::Entity;
use action::Action;
use direction::Direction;
use stats;
use ecs::traits::{Mutate, Query};
use world::EcsWorld;

fn pre_tick(_world: &mut EcsWorld) {

}

fn pre_tick_entity(_world: &mut EcsWorld, _entity: &Entity) {

}

pub fn run_action(world: &mut EcsWorld, entity: Entity, action: Action) {
    // Events are gathered up all at once. If an entity has already died in the
    // process of handling the previous events, it shouldn't get to run its
    // action.
    if !world.is_alive(entity) {
        return;
    }
    pre_tick(world);

    pre_tick_entity(world, &entity);
    run_entity_action(world, entity, action.clone());
    post_tick_entity(world, &entity);

    post_tick(world);
}

fn post_tick_entity(world: &mut EcsWorld, entity: &Entity) {
    if world.is_alive(*entity) {
        let delay = stats::formulas::calculate_delay(world, entity, 100);
        // debug!(entity.logger, "{} {}: delay {}, speed {}", entity.name(), name, delay, entity.speed);
        world.add_delay_for(entity, delay);
        world.after_entity_moved(*entity);
    }
}

fn post_tick(world: &mut EcsWorld) {
    // This has to go here because the entity's id hasn't been reinserted into
    // the world during the entity's post tick, meaning it can't be found when
    // it's attempted to be deleted.
    world.purge_dead();
}

fn try_move(world: &mut EcsWorld, entity: Entity, dir: Direction) {
    let new_pos = world.position(entity).expect("No entity position") + dir;
    if let Some(id) = world.mob_at(new_pos) {
        swing_at(world, &entity, &id);
    } else {
        world.set_entity_location(entity, new_pos);
    }
}

fn swing_at(world: &mut EcsWorld, attacker: &Entity, other: &Entity) {
    let damage;
    let evaded;
    {
        // if attacker.disposition == other.disposition {
        //     return;
        // }
        assert!(world.position(*attacker).unwrap().next_to(world.position(*other).unwrap()), "Tried swinging from out of range! (could be implemented)");
        evaded = stats::formulas::check_evasion(world, &attacker, &other);
        if evaded {
            // world.message("Evaded!".to_string());
            return;
        }
        damage = stats::formulas::calculate_damage(world, &attacker, &other);
    }
    println!("{}", damage);
}

fn run_entity_action(world: &mut EcsWorld, entity: Entity, action: Action) {
    match action {
        Action::Move(dir) => try_move(world, entity, dir),
        _ => (),
    }
}
