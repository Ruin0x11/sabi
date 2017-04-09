fn same_object<T>(a: &T, b: &T) -> bool {
    a as *const T == b as *const T
}

#[derive(Clone, Debug)]
struct ItemDesc {
    name: String,
    id: u32,
}

#[derive(Clone, Debug)]
struct Items<'a> {
    // capacity
    // weight limit
    items: Vec<Item<'a>>,
}

impl<'a> Items<'a> {
    pub fn new() -> Self {
        Items {
            items: Vec::new(),
        }
    }

    pub fn acquire(&mut self, item: Item<'a>) {
        for i in self.items.iter_mut() {
            if i.can_merge(&item) {
                i.merge(item);
                return;
            }
        }
        self.items.push(item);
    }

    pub fn find(&self, id: u32) -> Vec<&Item<'a>> {
        let mut results = Vec::new();
        for i in self.items.iter() {
            println!("{} - {}", i.desc.id, id);
            if i.desc.id == id {
                results.push(i);
            }
            results.extend(i.containing.find(id));
        }
        results
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }
}

#[derive(Clone, Debug)]
struct Item<'a> {
    pub desc: &'a ItemDesc,
    pub containing: Items<'a>,
    pub count: u32
}

impl<'a> Item<'a> {
    pub fn new(desc: &'a ItemDesc) -> Self {
        Item {
            desc: desc,
            count: 1,
            containing: Items::new(),
        }
    }

    pub fn is_container(&self) -> bool {
        self.containing.len() > 0
    }

    pub fn can_merge(&self, other: &Item) -> bool {
        if same_object(self, other) {
            return false;
        }

        if self.is_container() || other.is_container() {
            return false;
        }

        self.desc.id == other.desc.id
    }

    pub fn merge(&mut self, other: Item) {
        self.count += other.count;
    }

    pub fn split(&mut self, amount: u32) -> Option<Item> {
        if amount >= self.count || amount == 0 {
            return None;
        }

        let mut split_stack = self.clone();
        split_stack.count = amount;
        self.count -= amount;
        Some(split_stack)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_descs() -> Vec<ItemDesc> {
        vec![ItemDesc { name: "dream".to_string(), id: 1},
             ItemDesc { name: "kitchen knife".to_string(), id: 2},
             ItemDesc { name: "playchest".to_string(), id: 3}]
    }

    #[test]
    fn test_merge() {
        let descs = get_descs();
        let mut dream_a = Item::new(descs.get(0).unwrap());
        let dream_b = Item::new(descs.get(0).unwrap());

        assert_eq!(dream_a.can_merge(&dream_b), true);
        assert_eq!(dream_b.can_merge(&dream_a), true);
        assert_eq!(dream_a.can_merge(&dream_a), false);

        dream_a.merge(dream_b);
        assert_eq!(dream_a.count, 2);

        let knife = Item::new(descs.get(1).unwrap());

        assert_eq!(dream_a.can_merge(&knife), false)
    }

    #[test]
    fn test_split() {
        let descs = get_descs();
        let mut dream_a = Item::new(descs.get(0).unwrap());

        assert!(dream_a.split(1).is_none());

        dream_a.count = 3;

        assert!(dream_a.split(3).is_none());
        assert!(dream_a.split(0).is_none());
        assert!(dream_a.split(10).is_none());

        {
            let res = dream_a.split(1);
            assert!(res.is_some());
            assert_eq!(res.unwrap().count, 1);
        }
        assert_eq!(dream_a.count, 2);
    }

    #[test]
    fn test_find() {
        let descs = get_descs();
        let mut chest = Item::new(descs.get(2).unwrap());
        let mut upper_chest = Item::new(descs.get(2).unwrap());
        let mut dream = Item::new(descs.get(0).unwrap());
        dream.count = 42;

        chest.containing.acquire(dream);
        upper_chest.containing.acquire(chest);

        let results = upper_chest.containing.find(1);
        assert_eq!(results.len(), 1);
        let first = results.first().unwrap();
        assert_eq!(first.desc.id, 1);
        assert_eq!(first.count, 42);
    }
}
