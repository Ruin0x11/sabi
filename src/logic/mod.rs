mod action;
pub mod command;
pub mod entity;
mod debug_command;

pub use self::action::Action;
pub use self::command::{Command, CommandResult};

use calx_ecs::Entity;
use world::traits::*;
use world::World;

fn pre_tick(_world: &mut World) {

}

fn pre_tick_entity(_world: &mut World, _entity: Entity) {

}

pub fn run_action(world: &mut World, entity: Entity, action: Action) {
    // Events are gathered up all at once. If an entity has already died in the
    // process of handling the previous events, it shouldn't get to run its
    // action.
    if !world.is_alive(entity) {
        return;
    }

    pre_tick(world);

    pre_tick_entity(world, entity);
    action::run_entity_action(world, entity, action.clone());
    post_tick_entity(world, entity);

    post_tick(world);
}

fn post_tick_entity(world: &mut World, entity: Entity) {
    world.update_killed();

    if world.is_alive(entity) {
        world.after_entity_moved(entity);
    }
}

fn post_tick(_world: &mut World) {

}
