use std::collections::{hash_map, HashMap};

use testbed::item::{Item, ItemIdx, ItemLink};
use world::WorldPosition;

const MAX_WORLD_ITEMS: usize = 1000;

pub struct Items {
    pos_to_item_stacks: HashMap<WorldPosition, ItemIdx>,
    item_list: HashMap<ItemIdx, Item>,
}

impl Items {
    pub fn new() -> Self {
        Items {
            pos_to_item_stacks: HashMap::new(),
            item_list: HashMap::new(),
        }
    }

    fn get_free_idx(&self) -> usize {
        for i in 0..MAX_WORLD_ITEMS {
            if !self.item_list.contains_key(&i) {
                return i;
            }
        }
        panic!("Ran out of item space!");
    }

    pub fn add(&mut self, item: Item) -> ItemIdx {
        let next_idx = self.get_free_idx();
        self.item_list.insert(next_idx, item);
        next_idx
    }

    pub fn walk_stack<F, T>(&self, pos: &WorldPosition, mut callback: F) -> Option<T>
        where F: FnMut(ItemIdx) -> Option<T> {
        
        let idx = match self.pos_to_item_stacks.get(pos) {
            Some(i) => i.clone(),
            None    => return None,
        };

        let mut linked = idx;

        println!("Walking {}", linked);

        while let ItemLink::InStack(Some(next_link)) = self.get(&linked).link {
            let res = callback(linked);
            if res.is_some() {
                return res;
            }
            println!("next {:?}", linked);
            linked = next_link;
        }
        println!("dun");
        callback(linked)
    }

    /// Places the object at the top of the stack at the given world position.
    /// Returns the index in the items table that the item was given.
    pub fn place_in_world(&mut self, pos: WorldPosition, mut item: Item) -> ItemIdx {
        item.pos = pos;

        if self.pos_to_item_stacks.get(&pos).is_none() {
            item.link = ItemLink::InStack(None);
            let idx = self.add(item);
            println!("inserted {} at None", idx);
            self.pos_to_item_stacks.insert(pos, idx);
            return idx;
        }

        let idx = self.pos_to_item_stacks.get(&pos).unwrap().clone();
        let mergable_result = self.walk_stack(&pos, |linked| {
            let other = self.get(&linked);
            println!("other: {:?}", other);
            if other.can_merge(&item) {
                return Some(linked)
            }

            None
        });

        if let Some(mergable) = mergable_result {
            let other = self.get_mut(&mergable);
            other.merge(item);
            return mergable;
        }

        let next_idx = self.get_free_idx();

        item.link = ItemLink::InStack(Some(idx.clone()));
        self.item_list.insert(next_idx, item);
        self.pos_to_item_stacks.insert(pos, next_idx);

        next_idx
    }

    pub fn get_parent_in_stack(&self, pos: &WorldPosition,
                               target: &ItemIdx) -> Option<ItemIdx> {
        self.walk_stack(pos, |linked| {
            let next = self.get(&linked);
            if let ItemLink::InStack(Some(i)) = next.link {
                if i == *target {
                    return Some(linked)
                }
            }
            None
        })
    }

    pub fn remove(&mut self, idx: &ItemIdx) -> () {
        // First, check if the item is in a stack inside the world.
        let pos = self.get(idx).pos;
        if self.pos_to_item_stacks.get(&pos).is_some() {
            if let Some(parent_idx) = self.get_parent_in_stack(&pos, idx) {
                let next = self.get(idx).link.clone();
                let parent = self.get_mut(&parent_idx);
                parent.link = next;
            }
        }
        self.remove(idx);
    }

    pub fn get(&self, idx: &ItemIdx) -> &Item {
        self.item_list.get(idx).expect("Item with idx not found in world!")
    }

    pub fn get_mut(&mut self, idx: &ItemIdx) -> &mut Item {
        self.item_list.get_mut(idx).expect("Item with idx not found in world!")
    }

    pub fn at_pos(&self, pos: &WorldPosition) -> Option<&Item> {
        self.pos_to_item_stacks.get(pos).map(|idx| self.get(idx))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use testbed::item::*;

    lazy_static! {
        static ref DESCS: Vec<ItemDesc>  = vec![ItemDesc { name: "dream",
                                                           id: 1,
                                                           weight: 0.1,
                                                           container: false,
                                                           sprite: "none".to_string() },
                                                ItemDesc { name: "kitchen knife",
                                                           id: 2,
                                                           weight: 10.0,
                                                           container: false,
                                                           sprite: "none".to_string() },
                                                ItemDesc { name: "meatchest",
                                                           id: 3,
                                                           weight: 50.0,
                                                           container: true,
                                                           sprite: "none".to_string() }];
    }

    #[test]
    fn test_one() {
        let dream = Item::new(DESCS.get(0).unwrap());

        let mut items = Items::new();

        assert!(items.at_pos(&WorldPosition::new(0, 0)).is_none());
        items.place_in_world(WorldPosition::new(0, 0), dream);

        let res = items.at_pos(&WorldPosition::new(0, 0));

        assert!(res.is_some());
        assert_eq!(res.unwrap().link, ItemLink::InStack(None));
    }

    #[test]
    fn test_not_stacking() {
        let dream = Item::new(DESCS.get(0).unwrap());
        let knife = Item::new(DESCS.get(1).unwrap());

        let mut items = Items::new();

        let knife_idx = items.place_in_world(WorldPosition::new(0, 0), knife);
        items.place_in_world(WorldPosition::new(0, 0), dream);

        let res = items.at_pos(&WorldPosition::new(0, 0));

        assert!(res.is_some());
        assert_eq!(res.unwrap().link, ItemLink::InStack(Some(knife_idx)));
    }

    #[test]
    fn test_stacking() {
        let dream_a = Item::new(DESCS.get(0).unwrap());
        let dream_b = Item::new(DESCS.get(0).unwrap());

        let mut items = Items::new();

        let dream_a_idx = items.place_in_world(WorldPosition::new(0, 0), dream_a);
        let dream_b_idx = items.place_in_world(WorldPosition::new(0, 0), dream_b);

        assert_eq!(dream_a_idx, dream_b_idx);

        let res = items.at_pos(&WorldPosition::new(0, 0));

        assert!(res.is_some());
        assert_eq!(res.unwrap().link, ItemLink::InStack(None));
    }
}
