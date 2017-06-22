use std::collections::HashMap;
use std::collections::hash_map::Entry::*;
use self::PropErr::*;

use calx_ecs::Entity;

// TEMP: Experiment with different stat architectures and see what works.
//  - Static/baked in
//  - Key/value pairs
//    - Sets of required and optional properties?
//  - Lua tables?

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PropType {
    Bool(bool),
    Num(i64),
    Entity(Entity),
    String(String),
}

// This can be refactored to have type checking later.
// For now, Is[...] indicates bool, [...]Val indicates numeric.
// Using an enum instead of a string forces all used properties to be listed
// here, instead of being scattered in monster data/code.
//
// There is a tradeoff here between in-memory size/performance and convienience.
// The properties that are most important can be moved into the respective
// struct, and everything less important can live here.
#[derive(Debug)]
pub enum PropErr {
    NoSuchKey,
    WrongType,
}

/// Arbitrary key-value properties. Use when modeling things that don't fit
/// into the baked-in structs.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Properties {
    // poor-man's polymorphism
    props: HashMap<String, PropType>,
}

// The issues with this system are twofold:
//  - Keys can have arbitrary types, while typically a certain kind of property
//    always has a set type.
//  - One has to remember the correct type for a key when using 'get'.
impl Properties {
    pub fn new() -> Self {
        Properties {
            props: HashMap::new(),
        }
    }

    pub fn get<'a, T>(&'a self, key: &str) -> Result<&'a T, PropErr>
        where Properties: GetProp<T> {
        self.get_prop(key)
    }

    pub fn set<T>(&mut self, key: &str, val: T) -> Result<(), PropErr>
        where Properties: GetProp<T > {
        self.set_prop(key, val)
    }

    pub fn remove(&mut self, key: &str) {
        self.props.remove(key);
    }

    /// Convenience function to either get the value of a boolean property or
    /// return `false` if it doesn't exist.
    pub fn check_bool(&self, key: &str) -> bool {
        match self.get::<bool>(key) {
            Ok(val) => *val,
            Err(..) => false,
        }
    }
}

// Trying to create a HashMap with multiple types is harder than in an OO
// language. The types have to be wrapped in an enum and checked using the trait
// if they match by hand. The values inside can't be borrowed because it is the
// enum itself being borrowed, and returning a reference to the enum would make
// comparison operations harder, as opposed to unwrapping the value. The
// primitives inside each enum variant are inexpensive to copy, so hopefully it
// won't be an issue.
pub trait GetProp<T> {
    fn get_prop<'a>(&'a self, key: &str) -> Result<&'a T, PropErr>;
    fn set_prop(&mut self, key: &str, val: T) -> Result<(), PropErr>;
    fn remove_prop(&mut self, key: &str) -> Result<(), PropErr>;
}

macro_rules! make_get_set {
    ($ty:ty, $path:path) => {
        impl GetProp<$ty> for Properties {
            fn get_prop<'a>(&'a self, key: &str) -> Result<&'a $ty, PropErr> {
                if let Some(prop) = self.props.get(key) {
                    if let $path(ref val) = *prop {
                        Ok(val)
                    } else {
                        Err(WrongType)
                    }
                } else {
                    Err(NoSuchKey)
                }
            }

            fn set_prop(&mut self, key: &str, val: $ty) -> Result<(), PropErr> {
                match self.props.entry(key.to_string()) {
                    Occupied(mut v) => {
                        if let $path(..) = *v.get() {
                            *v.get_mut() = $path(val);
                            Ok(())
                        } else {
                            Err(WrongType)
                        }
                    }
                    Vacant(v) => {
                        v.insert($path(val));
                        Ok(())
                    }
                }
            }

            // This could return T in the future, given a type annotation.
            fn remove_prop(&mut self, key: &str) -> Result<(), PropErr> {
                match self.props.remove(key) {
                    Some(_) => Ok(()),
                    None    => Err(NoSuchKey)
                }
            }
        }
    }
}

make_get_set!(bool, PropType::Bool);
make_get_set!(i64, PropType::Num);
make_get_set!(Entity, PropType::Entity);
make_get_set!(String, PropType::String);

#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;

    #[test]
    fn test_set() {
        let mut props = Properties::new();
        let res = props.set("Test", 32);
        assert!(res.is_ok());
        {
            let hp = props.get::<i64>("Test").unwrap();
            assert_eq!(*hp, 32);
        }
        let res = props.set("Test", 128);
        assert!(res.is_ok());

        {
            let hp = props.get::<i64>("Test").unwrap();
            assert_eq!(*hp, 128);

            let res = props.get::<i64>("TestBool");
            assert!(res.is_err());
        }

        let res = props.set("Test", false);
        assert!(res.is_err());

        {
            let res = props.get::<bool>("Test");
            assert!(res.is_err());
        }
    }

    #[test]
    fn test_remove() {
        let mut props = Properties::new();
        let res = props.set("Test", 128);
        assert!(res.is_ok());

        props.remove("Test");

        let res = props.get::<i64>("Test");
        assert!(res.is_err());
    }

    #[bench]
    fn bench_set(b: &mut Bencher) {
        let mut props = Properties::new();
        for i in 0..10000 {
            props.set("Test", i);
        }
    }

    #[bench]
    fn bench_hashmap_set(b: &mut Bencher) {
        let mut map = HashMap::new();
        map.insert("happiness", 0);
        for i in 0..10000 {
            map.insert("happiness", i);
        }
    }
}
