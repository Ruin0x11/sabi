use calx_ecs::Entity;

use ai::AiTrigger;
use data::Walkability;
use ecs::traits::*;
use logic::entity::*;
use point::{Direction, Point};
use sound;
use stats;
use world::traits::*;
use world::{World, WorldPosition};

pub type ActionResult = Result<(), ()>;

#[derive(Clone, Debug)]
pub enum Action {
    Move(Direction),
    MoveOrAttack(Direction),
    Wait,
    SwitchPlaces(Entity),
    SwingAt(Entity),
    ShootAt(Entity),
    Pickup(Entity),
    Drop(Entity),
    Missile(Direction),

    Teleport(WorldPosition),
    TeleportUnchecked(WorldPosition),
}

pub fn run_entity_action(world: &mut World, entity: Entity, action: Action) -> ActionResult {
    match action {
        Action::MoveOrAttack(dir) => action_move_or_attack(world, entity, dir),
        Action::Move(dir) => action_move_entity(world, entity, dir),
        Action::Pickup(target) => action_pickup(world, entity, target),
        Action::SwitchPlaces(target) => action_switch_places(world, entity, target),
        Action::Drop(target) => action_drop(world, entity, target),
        Action::Teleport(pos) => action_try_teleport(world, entity, pos),
        Action::TeleportUnchecked(pos) => action_teleport_unchecked(world, entity, pos),
        Action::SwingAt(target) => action_swing_at(world, entity, target),
        Action::ShootAt(target) => action_shoot_at(world, entity, target),
        Action::Missile(dir) => action_missile(world, entity, dir),
        Action::Wait => Ok(()),
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
    world.move_entity(entity, dir).map_err(|_| ())?;

    if world.is_player(entity) {
        if let Some(on_ground) = world.entities_below(entity).first() {
            // NOTE: Duplicate from looking code
            format_mes!(world, entity, "%u <see> here {}.", on_ground.name_with_article(world));
        }
    }

    Ok(())
}

fn action_switch_places(world: &mut World, parent: Entity, target: Entity) -> ActionResult {
    let parent_pos = world.position(parent).expect("No entity position");
    let target_pos = world.position(target).expect("No entity position");

    world.place_entity(parent, target_pos);
    world.place_entity(target, parent_pos);
    format_mes!(world, parent, "%U <switch places> with {}.", target.name(world));
    Ok(())
}

fn action_pickup(world: &mut World, parent: Entity, target: Entity) -> ActionResult {
    world.place_entity_in(parent, target);
    format_mes!(world, parent, "%U <pick up> {}.", target.name(world));
    sound::play("pickup");
    Ok(())
}

fn action_drop(world: &mut World, entity: Entity, target: Entity) -> ActionResult {
    let pos = world.position(entity).unwrap();
    world.place_entity(target, pos);
    format_mes!(world, entity, "%U <drop> {}.", target.name(world));
    Ok(())
}

fn action_swing_at(world: &mut World, attacker: Entity, other: Entity) -> ActionResult {
    let mut damage;
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

        if world.ecs().healths.get_or_err(attacker).tp_full() {
            mes!(world, "Charge attack!");
            damage *= 4;

            world.ecs_mut().healths.get_mut_or_err(attacker).reset_tp();
        }
    }

    format_mes!(world, attacker, "%U <hit> {}! ({})", other.name(world), damage);
    hurt(world, other, attacker, damage);

    sound::play("damage");

    Ok(())
}

fn action_shoot_at(world: &mut World, attacker: Entity, other: Entity) -> ActionResult {
    let damage;
    {
        let missed = stats::formulas::check_evasion(world, attacker, other);
        if missed {
            mes!(world, "Miss.");
            return Ok(());
        }

        damage = stats::formulas::calculate_damage(world, attacker, other);
    }

    format_mes!(world, attacker, "%U <shoot at> {}! ({})", other.name(world), damage);
    hurt(world, other, attacker, damage);

    Ok(())
}

fn action_missile(world: &mut World, attacker: Entity, dir: Direction) -> ActionResult {
    mes!(world, "Firing in {}.", dir);

    Ok(())
}

fn hurt(world: &mut World, target: Entity, attacker: Entity, damage: u32) {
    world.ecs_mut().healths.map_mut(
        |h| {
            h.hurt(damage);
            h.adjust_tp(2);
        },
        target,
    );
    target.add_memory(AiTrigger::AttackedBy(attacker), world);

    world.ecs_mut()
         .healths
         .map_mut(|h| { h.adjust_tp(1); }, attacker);

    if target.is_dead(world) {
        format_mes!(world, attacker, "%U <kill> {}!", target.name(world));
        target.on_death(world);
    }
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
