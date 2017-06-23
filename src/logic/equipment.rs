use calx_ecs::Entity;

use logic::action::ActionResult;
use logic::entity::*;
use world::traits::*;
use world::World;

pub(super) fn equip(entity: Entity, equipment: Entity, slot: usize, world: &mut World) -> ActionResult {
    // After validating that the thing is equippable
    let should_equip = {
        let equipment_compo = world.ecs().equipments.get(entity).ok_or(())?;
        equipment_compo.can_equip(slot)
    };

    if should_equip {
        world.spatial.equip(equipment, entity, slot);
        format_mes!(world, entity, "%U <equip> {}", equipment.name(world));
    }

    Ok(())
}
