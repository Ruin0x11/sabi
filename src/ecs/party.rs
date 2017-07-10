use std::collections::HashMap;
use uuid::Uuid;

use calx_ecs::Entity;
use ecs::Loadout;
use logic::entity::*;
use world::World;
use world::traits::*;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum PartyMemberStatus {
    Active,
    Waiting(Loadout),
    Dead(Loadout),
}

fn member_name(uuid: &Uuid, status: &PartyMemberStatus, world: &World) -> String {
    match *status {
        PartyMemberStatus::Active => {
            world.entity_by_uuid(*uuid)
                 .map(|e| e.name(world))
                 .unwrap_or("(unknown)".to_string())
        },
        PartyMemberStatus::Waiting(ref loadout) |
        PartyMemberStatus::Dead(ref loadout) => {
            loadout.names
                   .clone()
                   .map(|n| n.name.clone())
                   .unwrap_or("(unknown)".to_string())
        },
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Party {
    members: HashMap<Uuid, PartyMemberStatus>,
}

impl Party {
    pub fn new() -> Self {
        Party { members: HashMap::new() }
    }

    pub fn add_member(&mut self, uuid: Uuid) {
        self.members.insert(uuid, PartyMemberStatus::Active);
    }

    pub fn set_status(&mut self, uuid: Uuid, status: PartyMemberStatus) {
        assert!(self.members.contains_key(&uuid));
        self.members.insert(uuid, status);
    }

    pub fn get_list(&self, world: &World) -> Vec<(Uuid, String)> {
        self.members
            .iter()
            .map(|(uuid, status)| (*uuid, member_name(uuid, status, world)))
            .collect()
    }

    pub fn contains_member(&self, entity: Entity, world: &World) -> bool {
        self.members.contains_key(&entity.uuid(world).unwrap())
    }

    pub fn active_uuids(&self) -> Vec<Uuid> {
        self.members
            .iter()
            .filter(|&(_, status)| is_member_active(status))
            .map(|(&uuid, _)| uuid)
            .collect()
    }
}

fn is_member_active(status: &PartyMemberStatus) -> bool {
    match *status {
        PartyMemberStatus::Active => true,
        _ => false,
    }
}
