use world::WorldPosition;

const ITEM_PILE_LIMIT: usize = 9999;

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

pub trait ItemCollection {
    fn acquire(&mut self, item: Item) -> ItemResult<()>;
    fn can_acquire(&self, item: &Item) -> bool;
    fn find(&self, id: u32) -> Vec<&Item>;
    fn len(&self) -> usize;
}

/// A pile of items on the ground.
pub type ItemPile = Vec<Item>;

impl ItemCollection for ItemPile {
    fn acquire(&mut self, item: Item) -> ItemResult<()> {
        if !self.can_acquire(&item) {
            return Err(CannotPutInContainer);
        }
        self.push(item);
        Ok(())
    }

    fn can_acquire(&self, _item: &Item) -> bool {
        self.len() < ITEM_PILE_LIMIT
    }

    fn find(&self, id: u32) -> Vec<&Item> {
        let mut results = Vec::new();
        for i in self.iter() {
            if i.desc.id == id {
                results.push(i);
            }
            results.extend(i.containing.find(id));
        }
        results
    }

    fn len(&self) -> usize {
        self.len()
    }
}

/// An collection of items inside a container, like a chest or actor's inventory.
#[derive(Clone, Debug)]
pub struct ItemContainer {
    capacity: usize,
    weight_limit: f32,
    items: Vec<Item>,

    cached_weight: f32,
}

impl ItemContainer {
    pub fn new() -> Self {
        ItemContainer {
            capacity: 100,
            weight_limit: 100.0,
            items: Vec::new(),

            cached_weight: 0.0,
        }
    }

    pub fn recalculate_weight(&mut self) -> f32 {
        let mut total = 0.0;
        for i in self.items.iter_mut() {
            i.containing.recalculate_weight();
            total += i.weight();
        }
        self.cached_weight = total;
        total
    }
}

impl ItemCollection for ItemContainer {
    fn can_acquire(&self, item: &Item) -> bool {
        if self.items.len() >= self.capacity {
            return false
        }
        if self.cached_weight + item.weight() > self.weight_limit {
            return false
        }

        true
    }

    fn acquire(&mut self, item: Item) -> ItemResult<()> {
        for i in self.items.iter_mut() {
            if i.can_merge(&item) {
                i.merge(item);
                return Ok(());
            }
        }

        if !self.can_acquire(&item) {
            return Err(CannotPutInContainer)
        }

        self.items.push(item);

        self.recalculate_weight();

        Ok(())
    }

    fn find(&self, id: u32) -> Vec<&Item> {
        self.items.find(id)
    }

    fn len(&self) -> usize {
        self.items.len()
    }
}

#[derive(Clone, Debug)]
pub struct Item {
    pub desc: &'static ItemDesc,
    pub custom_name: Option<String>,
    pub containing: ItemContainer,
    pub count: u32,

    pub pos: WorldPosition,

    pub link: ItemLink,
    pub loc: Location,
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum ItemLink {
    InStack(Option<ItemIdx>),

    // Used for finding out what in the player's inventory is equipped where.
    InInventory(usize),
    Nowhere,
}

#[derive(Clone, Debug)]
pub enum Location {
    World(WorldPosition),
    YourInventory,
    OtherInventory,
}

impl Item {
    pub fn new(desc: &'static ItemDesc) -> Self {
        Item {
            desc: desc,
            custom_name: None,
            count: 1,
            containing: ItemContainer::new(),
            pos: WorldPosition::new(0, 0),
            link: ItemLink::Nowhere,
            loc: Location::World(WorldPosition::new(0, 0)),
        }
    }

    pub fn get_pos(&self) -> WorldPosition {
        self.pos
    }

    pub fn sanity_check(&self) {
        if self.count == 0 {
            panic!("Item count is zero! {:?}", self.desc);
        }

        if self.is_container() && self.count != 1 {
            panic!("Item is container, but has a count! {:?}", self.desc);
        }

        if !self.is_container() && self.containing.len() > 0 {
            panic!("Item is not a container, but contains items! {:?}", self.desc);
        }
    }

    pub fn weight(&self) -> f32 {
        self.desc.weight * self.count as f32
            + self.containing.cached_weight
    }

    pub fn format_name(&self) -> String {
        let body = match self.custom_name {
            Some(ref cust) => format!("{} \"{}\"", self.desc.name, cust),
            None       => self.desc.name.clone().to_string(),
        };
        if self.count == 1 {
            body
        } else {
            format!("{}x {}", self.count, body)
        }
    }

    pub fn set_name(&mut self, name: String) {
        if name.is_empty() {
            self.custom_name = None;
        } else {
            self.custom_name = Some(name);
        }
    }

    pub fn is_container(&self) -> bool {
        self.desc.container
    }

    pub fn can_merge(&self, other: &Item) -> bool {
        if same_object(self, other) {
            return false;
        }

        if self.is_container() || other.is_container() {
            return false;
        }

        if self.custom_name.is_some() || other.custom_name.is_some() {
            if self.custom_name != other.custom_name {
                return false
            }
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
    fn test_merge() {
        let mut dream_a = Item::new(DESCS.get(0).unwrap());
        let dream_b = Item::new(DESCS.get(0).unwrap());

        assert_eq!(dream_a.can_merge(&dream_b), true);
        assert_eq!(dream_b.can_merge(&dream_a), true);
        assert_eq!(dream_a.can_merge(&dream_a), false);

        dream_a.merge(dream_b);
        assert_eq!(dream_a.count, 2);

        let knife = Item::new(DESCS.get(1).unwrap());

        assert_eq!(dream_a.can_merge(&knife), false)
    }

    #[test]
    fn test_set_name() {
        let mut argh = Item::new(DESCS.get(0).unwrap());
        argh.set_name("argh".to_string());
        assert_eq!(argh.format_name(), "dream \"argh\"");
        argh.set_name("".to_string());
        assert_eq!(argh.format_name(), "dream");
    }

    #[test]
    fn test_named_merge() {
        let mut trance = Item::new(DESCS.get(0).unwrap());
        let mut ennui = Item::new(DESCS.get(0).unwrap());
        trance.set_name("Trance".to_string());
        ennui.set_name("Ennui".to_string());
        assert_eq!(trance.can_merge(&ennui), false);

        ennui.set_name("Trance".to_string());
        assert_eq!(trance.can_merge(&ennui), true);
    }

    #[test]
    fn test_container_merge() {
        let chest = Item::new(DESCS.get(2).unwrap());
        let upper_chest = Item::new(DESCS.get(2).unwrap());
        assert_eq!(chest.can_merge(&upper_chest), false);
    }

    #[test]
    fn test_split() {
        let mut dream_a = Item::new(DESCS.get(0).unwrap());

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
        let mut chest = Item::new(DESCS.get(2).unwrap());
        let mut upper_chest = Item::new(DESCS.get(2).unwrap());
        let mut dream = Item::new(DESCS.get(0).unwrap());
        dream.count = 42;

        chest.containing.acquire(dream).unwrap();
        upper_chest.containing.acquire(chest).unwrap();

        let results = upper_chest.containing.find(1);
        assert_eq!(results.len(), 1);
        let first = results.first().unwrap();
        assert_eq!(first.desc.id, 1);
        assert_eq!(first.count, 42);
    }

    #[test]
    fn test_container_weights() {

        let dream = Item::new(DESCS.get(0).unwrap());
        let knife = Item::new(DESCS.get(1).unwrap());
        let chest = Item::new(DESCS.get(2).unwrap());
        let other_chest = chest.clone();

        let mut container = ItemContainer::new();
        assert_eq!(container.weight_limit, 100.0);
        assert_eq!(container.cached_weight, 0.0);

        container.acquire(dream).unwrap();
        assert_eq!(container.cached_weight, 0.1);

        container.acquire(knife).unwrap();
        assert_eq!(container.cached_weight, 10.1);

        container.acquire(chest).unwrap();
        assert_eq!(container.cached_weight, 60.1);

        let res = container.acquire(other_chest);
        assert!(res.is_err(), "limit: {}, weight: {}", container.weight_limit, container.cached_weight);
        assert_eq!(container.cached_weight, 60.1);
    }

    #[test]
    fn test_weights() {

        let dream =     Item::new(DESCS.get(0).unwrap());
        let knife =     Item::new(DESCS.get(1).unwrap());
        let mut chest = Item::new(DESCS.get(2).unwrap());
        let mut other_chest = chest.clone();

        assert_eq!(chest.weight(), 50.0);
        assert_eq!(chest.containing.weight_limit, 100.0);

        chest.containing.acquire(knife).unwrap();
        chest.containing.acquire(dream).unwrap();
        assert_eq!(chest.weight(), 60.1);

        other_chest.containing.acquire(chest).unwrap();
        assert_eq!(other_chest.weight(), 110.1);
    }
}
