use world::*;

type Spatial<T> = HashMap<WorldPosition, T>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all() {
        let s = ItemSpatial::new();
        let c = Item::new();
        let i = Item::new();
        c.put(i);
        s.put(c);
    }
}
