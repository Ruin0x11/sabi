use calx_ecs::Entity;
use data::Walkability;
use ecs::traits::*;
use logic::command::CommandResult;
use logic::entity;
use lua;
use point::{Direction, Point};
use prefab;
use stats;
use world::traits::*;
use world::{EcsWorld, WorldPosition};

#[derive(Clone, Debug)]
pub enum Action {
    Move(Direction),
    MoveOrAttack(Direction),
    Wait,
    SwingAt(Entity),

    TestScript,

    Teleport(Point),
    TeleportUnchecked(Point),
}

pub fn run_entity_action(world: &mut EcsWorld, entity: Entity, action: Action) -> CommandResult {
    match action {
        Action::MoveOrAttack(dir)      => move_or_attack(world, entity, dir),
        Action::Move(dir)              => world.move_entity(entity, dir),
        Action::TestScript              => {
            match lua::with_mut(|l| prefab::map_from_prefab(l, "prefab")) {
                Ok(_)  => Ok(()),
                Err(e) => {lua::log::lua_log_error(format!("{:?}", e)); Err(())},
            }
        },
        Action::Teleport(pos)          => try_teleport(world, entity, pos),
        Action::TeleportUnchecked(pos) => {
            world.set_entity_location(entity, pos);
            Ok(())
        }
        Action::SwingAt(target)        => swing_at(world, entity, target),
        _ => Err(()),
    }
}

fn move_or_attack(world: &mut EcsWorld, entity: Entity, dir: Direction) -> CommandResult {
    let new_pos = world.position(entity).expect("No entity position") + dir;
    if let Some(id) = world.mob_at(new_pos) {
        swing_at(world, entity, id)
    } else {
        world.move_entity(entity, dir)
    }
}

fn swing_at(world: &mut EcsWorld, attacker: Entity, other: Entity) -> CommandResult {
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

    mes!(world, "The {} hits the {}! ({})", a=entity::name(attacker, world), b=entity::name(other, world), c=damage);

    if entity::is_dead(other, world) {
        mes!(world, "The {} kills the {}!", a=entity::name(attacker, world), b=entity::name(other, world));
    }

    Ok(())
}

fn try_teleport(world: &mut EcsWorld, entity: Entity, pos: WorldPosition) -> CommandResult {
    if world.can_walk(pos, Walkability::MonstersBlocking) {
        world.set_entity_location(entity, pos);
        Ok(())
    } else {
        Err(())
    }
}
