mod effect;

pub use self::effect::ItemEffect;

use std::collections::HashSet;

use calx_ecs::Entity;

use world::{World, WorldPosition};
use world::traits::*;

fn same_object<T>(a: &T, b: &T) -> bool {
    a as *const T == b as *const T
}

pub type ItemIdx = usize;

#[derive(Debug)]
pub enum ItemErr {
    CannotPutInContainer,
}

pub use self::ItemErr::*;

pub type ItemResult<T> = Result<T, ItemErr>;

#[derive(Clone, Debug)]
pub struct ItemDesc {
    pub name: &'static str,
    pub weight: f32,
    pub id: u32,
    pub container: bool,
    pub sprite: String,
}

/// An collection of items inside a container, like a chest or actor's inventory.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ItemContainer {
    pub capacity: usize,
    pub weight_limit: f32,
    pub items: HashSet<Entity>,

    pub cached_weight: f32,
}

impl ItemContainer {
    pub fn new() -> Self {
        ItemContainer {
            capacity: 100,
            weight_limit: 100.0,
            items: HashSet::new(),

            cached_weight: 0.0,
        }
    }

    pub fn recalculate_weight(&mut self, world: &World) -> f32 {
        let mut total = 0.0;
        for entity in self.items.iter() {
            // i.containing.recalculate_weight();
            let item = world.ecs().items.get(*entity).unwrap();
            total += item.weight();
        }
        self.cached_weight = total;
        total
    }
}

// impl ItemContainer {
//     pub fn can_acquire(&self, item: &Entity, world: &World) -> bool {
//         let item_compo = world.ecs().items.get(*item).unwrap();

//         if self.items.len() >= self.capacity {
//             return false
//         }
//         if self.cached_weight + item_compo.weight() > self.weight_limit {
//             return false
//         }

//         true
//     }

//     pub fn acquire(&mut self, item: Entity, world: &mut World) -> ItemResult<()> {
//         let item_compo = world.ecs().items.get(item).unwrap();
//         for i in self.items.iter() {
//             let i_compo = world.ecs_mut().items.get_mut(*i).unwrap();
//             if i_compo.can_merge(&item_compo) {
//                 i_compo.merge(item_compo);
//                 return Ok(());
//             }
//         }

//         if !self.can_acquire(&item, world) {
//             return Err(CannotPutInContainer)
//         }

//         assert!(!self.items.contains(&item));
//         self.items.insert(item);

//         self.recalculate_weight(world);

//         Ok(())
//     }

//     pub fn remove(&mut self, entity: Entity) -> ItemResult<()> {
//         self.items.remove(&entity);
//         Ok(())
//     }

//     pub fn len(&self) -> usize {
//         self.items.len()
//     }
// }
