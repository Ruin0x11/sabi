use calx_ecs::Entity;
use data::Walkability;
use ecs::traits::*;
use logic::command::CommandResult;
use logic::entity::EntityQuery;
use lua;
use point::{Direction, Point};
use prefab;
use stats;
use world::traits::*;
use world::{EcsWorld, WorldPosition};

pub type ActionResult = Result<(), ()>;

#[derive(Clone, Debug)]
pub enum Action {
    Move(Direction),
    MoveOrAttack(Direction),
    Wait,
    SwingAt(Entity),
    Pickup,

    TestScript,

    Teleport(Point),
    TeleportUnchecked(Point),
}

pub fn run_entity_action(world: &mut EcsWorld, entity: Entity, action: Action) -> ActionResult {
    match action {
        Action::MoveOrAttack(dir)      => action_move_or_attack(world, entity, dir),
        Action::Move(dir)              => action_move_entity(world, entity, dir),
        Action::TestScript             => action_test_script(),
        Action::Teleport(pos)          => action_try_teleport(world, entity, pos),
        Action::TeleportUnchecked(pos) => action_teleport_unchecked(world, entity, pos),
        Action::SwingAt(target)        => action_swing_at(world, entity, target),
        _ => Err(()),
    }
}

fn action_teleport_unchecked(world: &mut EcsWorld, entity: Entity, pos: Point) -> ActionResult {
    world.set_entity_location(entity, pos);
    Ok(())
}

fn action_test_script() -> ActionResult {
    Ok(())
}

fn action_move_or_attack(world: &mut EcsWorld, entity: Entity, dir: Direction) -> ActionResult {
    let new_pos = world.position(entity).expect("No entity position") + dir;
    if let Some(id) = world.mob_at(new_pos) {
        action_swing_at(world, entity, id)
    } else {
        action_move_entity(world, entity, dir)
    }
}

fn action_move_entity(world: &mut EcsWorld, entity: Entity, dir: Direction) -> ActionResult {
    world.move_entity(entity, dir).map_err(|_| ())
}

fn action_swing_at(world: &mut EcsWorld, attacker: Entity, other: Entity) -> ActionResult {
    let damage;
    {
        if !world.position(attacker).unwrap().is_next_to(world.position(other).unwrap()) {
            return Err(());
        }

        let missed = stats::formulas::check_evasion(world, &attacker, &other);
        if missed {
            mes!(world, "Miss.");
            return Ok(());
        }

        damage = stats::formulas::calculate_damage(world, &attacker, &other);
    }
    world.ecs_mut().healths.map_mut(|h| h.hurt(damage), other);

    mes!(world, "The {} hits the {}! ({})", a=attacker.name(world), b=other.name(world), c=damage);

    if other.is_dead(world) {
        mes!(world, "The {} kills the {}!", a=attacker.name(world), b=other.name(world));
    }

    Ok(())
}

fn action_try_teleport(world: &mut EcsWorld, entity: Entity, pos: WorldPosition) -> ActionResult {
    if world.can_walk(pos, Walkability::MonstersBlocking) {
        mes!(world, "Suddenly, the {} disappears.", a=entity.name(world));
        world.set_entity_location(entity, pos);
        Ok(())
    } else {
        Err(())
    }
}
