use std::collections::Bound::{Included, Unbounded};
use std::collections::{BTreeMap, btree_map};
use serde;
use calx_ecs::Entity;

use point::Point;

use self::Place::*;

#[derive(Copy, Eq, PartialEq, Clone, PartialOrd, Ord, Debug, Serialize, Deserialize)]
pub enum Place {
    Unloaded(Point),
    At(Point),
    In(Entity),
}

/// Spatial index for game entities
pub struct Spatial {
    place_to_entities: BTreeMap<Place, Vec<Entity>>,
    entity_to_place: BTreeMap<Entity, Place>,
}

impl Spatial {
    pub fn new() -> Spatial {
        Spatial {
            place_to_entities: BTreeMap::new(),
            entity_to_place: BTreeMap::new(),
        }
    }

    fn insert(&mut self, entity: Entity, place: Place) {
        // Remove the entity from its old position.
        self.single_remove(entity);

        self.entity_to_place.insert(entity, place);
        if let Some(entities) = self.place_to_entities.get_mut(&place) {
            entities.push(entity);
            return;
        };
        // Didn't return above, that means this location isn't indexed
        // yet and needs a brand new container. (Can't do this in match
        // block because borrows.)
        self.place_to_entities.insert(place, vec![entity]);
    }

    /// Insert an entity into space.
    pub fn insert_at(&mut self, entity: Entity, loc: Point) { self.insert(entity, At(loc)); }

    /// Return whether the parent entity or an entity contained in the parent
    /// entity contains entity e.
    pub fn contains(&self, parent: Entity, e: Entity) -> bool {
        match self.entity_to_place.get(&e) {
            Some(&In(p)) if p == parent => true,
            Some(&In(p)) => self.contains(parent, p),
            _ => false,
        }
    }

    /// Insert an entity into container.
    pub fn insert_in(&mut self, e: Entity, parent: Entity) {
        assert!(!self.contains(e, parent),
                "Trying to create circular containment");
        self.insert(e, In(parent));
    }

    /// Remove an entity from the local structures but do not pop out its
    /// items. Unless the entity is added back in or the contents are handled
    /// somehow, this will leave the spatial index in an inconsistent state.
    fn single_remove(&mut self, entity: Entity) {
        if !self.entity_to_place.contains_key(&entity) {
            return;
        }

        let &place = &self.entity_to_place[&entity];
        self.entity_to_place.remove(&entity);

        {
            let entities = self.place_to_entities.get_mut(&place).unwrap();
            assert!(!entities.is_empty());
            if entities.len() > 1 {
                // More than one entity present, remove this one, keep the
                // rest.
                for i in 0..entities.len() {
                    if entities[i] == entity {
                        entities.swap_remove(i);
                        return;
                    }
                }
                panic!("Entity being removed from place it's not in");
            } else {
                // This was the only entity in the location.
                // Drop the entry for this location from the index.
                // (Need to drop out of scope for borrows reasons)
                assert_eq!((*entities)[0],  entity);
            }
        }
        // We only end up here if we need to clear the container for the
        // location.
        self.place_to_entities.remove(&place);
    }

    /// Remove an entity from the space. Entities contained in the entity will
    /// also be removed from the space.
    pub fn remove(&mut self, e: Entity) {
        // Remove the contents
        for &content in self.entities_in(e).iter() {
            self.remove(content);
        }
        self.single_remove(e);
    }

    pub fn freeze(&mut self, e: Entity) {
        if !self.entity_to_place.contains_key(&e) {
            return;
        }

        if let Some(At(pos)) = self.get(e) {
            self.insert(e, Unloaded(pos));
        }
    }

    pub fn unfreeze(&mut self, e: Entity) {
        if !self.entity_to_place.contains_key(&e) {
            return;
        }

        if let Some(Unloaded(pos)) = self.get(e) {
            self.insert(e, At(pos));
        }
    }

    fn entities(&self, p: Place) -> Vec<Entity> {
        match self.place_to_entities.get(&p) {
            None => vec![],
            Some(v) => v.clone(),
        }
    }

    pub fn iter(&self) -> btree_map::Iter<Entity, Place> { self.entity_to_place.iter() }

    /// List entities at a location.
    pub fn entities_at(&self, loc: Point) -> Vec<Entity> { self.entities(At(loc)) }

    /// List entities in a container.
    pub fn entities_in(&self, parent: Entity) -> Vec<Entity> {
        println!("ENTITIES IN: {:?}", self.place_to_entities);
        self.place_to_entities.range((Included(&In(parent)), Unbounded))
        // Consume the contingent elements for the parent container.
             .take_while(|&(ref k, _)| if let &&In(ref p) = k { *p == parent } else { false })
             .flat_map(|(_, v)| v.iter())
             .cloned()
             .collect()
    }

    /// Return the place of an entity if the entity is present in the space.
    pub fn get(&self, e: Entity) -> Option<Place> {
        self.entity_to_place.get(&e).cloned()
    }

    /// Flatten to an easily serializable vector.
    fn dump(&self) -> Vec<Elt> {
        let mut ret = vec![];
        for (&e, &loc) in self.entity_to_place.iter() {
            ret.push(Elt(e, loc));
        }
        ret
    }

    /// Flattens the children of an entity.
    fn dump_containing(&self, parent: Entity) -> Vec<Elt> {
        let mut ret = vec![];

        for child in self.entities_in(parent) {
            ret.push(Elt(child, In(parent)));
        }

        ret
    }

    /// Construct from the serialized vector.
    fn slurp(dump: Vec<Elt>) -> Spatial {
        let mut ret = Spatial::new();

        for &Elt(e, loc) in dump.iter() {
            ret.insert(e, loc);
        }
        ret
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Elt(Entity, Place);

impl serde::Serialize for Spatial {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        self.dump().serialize(s)
    }
}

impl serde::Deserialize for Spatial {
    fn deserialize<D: serde::Deserializer>(d: D) -> Result<Self, D::Error> {
        Ok(Spatial::slurp(serde::Deserialize::deserialize(d)?))
    }
}

#[cfg(test)]
mod test {
    use super::{Place, Spatial};
    use ecs::Ecs;
    use point::Point;

    #[cfg(never)]
    #[test]
    fn test_place_adjacency() {
        let mut ecs = Ecs::new();
        let e1 = ecs.make();
        let e2 = ecs.make();

        // Test that the Place type gets a lexical ordering where elements in
        // the same parent entity get sorted next to each other, and that None
        // is the minimum value for the slot option.
        //
        // This needs to be right for the containment logic to function, but
        // it's not obvious which way the derived lexical order sorts, so put
        // an unit test here to check it out.
        let mut places = vec![
            Place::In(e1, Some(Slot::Melee)),
            Place::In(e2, None),
            Place::In(e1, Some(Slot::Ranged)),
            Place::In(e1, None),
        ];

        places.sort();
        assert_eq!(places,
                   vec![
                       Place::In(e1, None),
                       Place::In(e1, Some(Slot::Melee)),
                       Place::In(e1, Some(Slot::Ranged)),
                       Place::In(e2, None),
                   ]);
    }

    #[test]
    fn test_serialization() {
        use bincode;

        let mut ecs = Ecs::new();
        let e1 = ecs.make();
        // let e2 = ecs.make();

        let mut spatial = Spatial::new();
        let p1 = Place::At(Point::new(10, 10));
        // let p2 = Place::In(e1, None);
        spatial.insert(e1, p1);
        // spatial.insert(e2, p2);

        let saved = bincode::serialize(&spatial, bincode::Infinite)
            .expect("Spatial serialization failed");
        let spatial2: Spatial = bincode::deserialize(&saved)
            .expect("Spatial deserialization failed");

        assert_eq!(spatial2.get(e1), Some(p1));
        // assert_eq!(spatial2.get(e2), Some(p2));
    }

    #[test]
    fn test_freeze_unfreeze() {
        let mut ecs = Ecs::new();
        let e1 = ecs.make();

        let mut spatial = Spatial::new();
        let p1 = Place::At(Point::new(10, 10));
        spatial.insert(e1, p1);

        spatial.freeze(e1);
        assert_eq!(spatial.get(e1), Some(Place::Unloaded(Point::new(10, 10))));
        assert_eq!(spatial.entity_to_place.len(), 1);
        spatial.unfreeze(e1);
        assert_eq!(spatial.get(e1), Some(p1))
    }
}
