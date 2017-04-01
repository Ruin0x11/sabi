use std::collections::HashMap;
use std::collections::hash_map::Entry::*;
use self::PropErr::*;

pub use self::PropName::*;

// TEMP: Experiment with different stat architectures and see what works.
//  - Static/baked in
//  - Key/value pairs
//    - Sets of required and optional properties?
//  - Lua tables?

enum Prop {
    Bool(bool),
    Num(i32),
}

// This can be refactored to have type checking later.
// For now, Is[...] indicates bool, [...]Val indicates numeric.
// Using an enum instead of a string forces all used properties to be listed
// here, instead of being scattered in monster data/code.
#[derive(Eq, PartialEq, Hash, Clone, Debug)]
pub enum PropName {
    Happiness,
    Sadness,
}

#[derive(Debug)]
pub enum PropErr {
    NoSuchKey,
    WrongType,
}

/// Arbitrary key-value properties. Use when modeling things that don't fit
/// into the baked-in structs.
pub struct Properties {
    // poor-man's polymorphism
    props: HashMap<PropName, Prop>,
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

    pub fn get<T>(&self, key: PropName) -> Result<T, PropErr>
        where Properties: GetProp<T, PropKey=PropName> {
        self.get_prop(key)
    }

    pub fn set<T>(&mut self, key: PropName, val: T) -> Result<(), PropErr>
        where Properties: GetProp<T, PropKey=PropName> {
        self.set_prop(key, val)
    }

    pub fn remove(&mut self, key: PropName) -> () {
        self.props.remove(&key);
    }
}

// Trying to create a HashMap with multiple types is harder than in an OO
// language. The types have to be wrapped in an enum and checked using the trait
// if they match by hand. The values inside can't be borrowed because it is the
// enum itself being borrowed, and returning a reference to the enum would make
// comparison operations harder, as opposed to unwrapping the value. The
// primitives inside each enum variant are inexpensive to copy, so hopefully it
// won't be an issue.
// TEMP: This is just a dirty hack.
pub trait GetProp<T> {
    type PropKey;
    fn get_prop(&self, key: Self::PropKey) -> Result<T, PropErr>;
    fn set_prop(&mut self, key: Self::PropKey, val: T) -> Result<(), PropErr>;
    fn remove_prop(&mut self, key: Self::PropKey) -> Result<(), PropErr>;
}

macro_rules! make_get_set {
    ($ty:ty, $path:path) => {
        impl GetProp<$ty> for Properties {
            type PropKey = PropName;
            fn get_prop(&self, key: PropName) -> Result<$ty, PropErr> {
                if let Some(prop) = self.props.get(&key) {
                    if let $path(val) = *prop {
                        Ok(val)
                    } else {
                        Err(WrongType)
                    }
                } else {
                    Err(NoSuchKey)
                }
            }

            fn set_prop(&mut self, key: PropName, val: $ty) -> Result<(), PropErr> {
                match self.props.entry(key) {
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
            fn remove_prop(&mut self, key: PropName) -> Result<(), PropErr> {
                match self.props.remove(&key) {
                    Some(_) => Ok(()),
                    None      => Err(NoSuchKey)
                }
            }
        }
    }
}

make_get_set!(bool, Prop::Bool);
make_get_set!(i32,  Prop::Num);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set() {
        let mut props = Properties::new();
        let res = props.set(Happiness, 32);
        assert!(res.is_ok());
        let hp = props.get::<i32>(Happiness).unwrap();
        assert_eq!(hp, 32);
        let res = props.set(Happiness, 128);
        assert!(res.is_ok());
        let hp = props.get::<i32>(Happiness).unwrap();
        assert_eq!(hp, 128);

        let res = props.get::<i32>(Sadness);
        assert!(res.is_err());

        let res = props.set(Happiness, false);
        assert!(res.is_err());

        let res = props.get::<bool>(Happiness);
        assert!(res.is_err());

    }

    #[test]
    fn test_remove() {
        let mut props = Properties::new();
        let res = props.set(Happiness, 128);
        assert!(res.is_ok());

        props.remove(Happiness);

        let res = props.get::<i32>(Happiness);
        assert!(res.is_err());
    }

    #[cfg(never)]
    #[bench]
    fn bench_set(b: &mut Bencher) {
        let props = Properties::new();
        for i in 0..10000 {
            props.set(Happiness, i);
        }
    }

    #[cfg(never)]
    #[bench]
    fn bench_hashmap_set(b: &mut Bencher) {
        let map = HashMap::new();
        map.insert("happiness", 0);
        for i in 0..10000 {
            map.insert("happiness", i);
        }
    }
}
