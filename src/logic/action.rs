use calx_ecs::Entity;
use data::Walkability;
use ecs::traits::*;
use logic::entity::EntityQuery;
use point::{Direction, Point};
use stats;
use world::traits::*;
use world::{World, WorldPosition};

pub type ActionResult = Result<(), ()>;

#[derive(Clone, Debug)]
pub enum Action {
    Move(Direction),
    MoveOrAttack(Direction),
    Wait,
    SwingAt(Entity),
    Pickup(Entity),
    Drop(Entity),

    Teleport(WorldPosition),
    TeleportUnchecked(WorldPosition),
}

pub fn run_entity_action(world: &mut World, entity: Entity, action: Action) -> ActionResult {
    match action {
        Action::MoveOrAttack(dir) => action_move_or_attack(world, entity, dir),
        Action::Move(dir) => action_move_entity(world, entity, dir),
        Action::Pickup(target) => action_pickup(world, entity, target),
        Action::Drop(target) => action_drop(world, entity, target),
        Action::Teleport(pos) => action_try_teleport(world, entity, pos),
        Action::TeleportUnchecked(pos) => action_teleport_unchecked(world, entity, pos),
        Action::SwingAt(target) => action_swing_at(world, entity, target),
        _ => Err(()),
    }
}

fn action_teleport_unchecked(world: &mut World, entity: Entity, pos: Point) -> ActionResult {
    world.place_entity(entity, pos);
    Ok(())
}

fn action_move_or_attack(world: &mut World, entity: Entity, dir: Direction) -> ActionResult {
    let new_pos = world.position(entity).expect("No entity position") + dir;
    if let Some(id) = world.mob_at(new_pos) {
        action_swing_at(world, entity, id)
    } else {
        action_move_entity(world, entity, dir)
    }
}

fn action_move_entity(world: &mut World, entity: Entity, dir: Direction) -> ActionResult {
    world.move_entity(entity, dir).map_err(|_| ())
}

fn action_swing_at(world: &mut World, attacker: Entity, other: Entity) -> ActionResult {
    let damage;
    {
        if !world.position(attacker)
                 .unwrap()
                 .is_next_to(world.position(other).unwrap())
        {
            return Err(());
        }

        let missed = stats::formulas::check_evasion(world, attacker, other);
        if missed {
            mes!(world, "Miss.");
            return Ok(());
        }

        damage = stats::formulas::calculate_damage(world, attacker, other);
    }
    world.ecs_mut().healths.map_mut(|h| h.hurt(damage), other);

    format_mes!(world, attacker, "%U <hit> {}! ({})", a = other.name(world), b = damage);

    if other.is_dead(world) {
        format_mes!(world, attacker, "%U <kill> {}! ({})", a = other.name(world), b = damage);
    }

    Ok(())
}

fn action_pickup(world: &mut World, parent: Entity, target: Entity) -> ActionResult {
    world.place_entity_in(parent, target);
    mes!(world, "{} picks up {}.", a = parent.name(world), b = target.name(world));
    Ok(())
}

fn action_drop(world: &mut World, entity: Entity, target: Entity) -> ActionResult {
    let pos = world.position(entity).unwrap();
    world.place_entity(target, pos);
    format_mes!(world, entity, "%U <drop> {}.", a = target.name(world));
    Ok(())
}

fn action_try_teleport(world: &mut World, entity: Entity, pos: WorldPosition) -> ActionResult {
    if world.can_walk(pos, Walkability::MonstersBlocking) {
        format_mes!(world, entity, "Suddenly, %U <disappear>.");
        world.place_entity(entity, pos);
        Ok(())
    } else {
        Err(())
    }
}
