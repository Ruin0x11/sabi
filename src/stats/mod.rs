use std::collections::HashMap;
use std::collections::hash_map::Entry::*;
use self::PropErr::*;

// TEMP: Experiment with different stat architectures and see what works.
//  - Static/baked in
//  - Key/value pairs
//    - Sets of required and optional properties?
//  - Lua tables?

enum Prop {
    Bool(bool),
    Num(i32),
}

#[derive(Debug)]
enum PropErr {
    NoSuchKey,
    WrongType,
}

struct Properties {
    props: HashMap<String, Prop>,
}

// The issues with this system are threefold:
//  - Keys can have arbitrary types, while typically a certain kind of property
//    always has a set type.
//  - One has to remember the correct type for a key when using 'get'.
//  - It's possible to look for a key that shouldn't be there. There ought to be
//    a list of valid keys.
impl Properties {
    pub fn new() -> Self {
        Properties {
            props: HashMap::new(),
        }
    }

    pub fn get<T>(&self, key: &str) -> Result<T, PropErr>
        where Properties: GetProp<T> {
        self.get_prop(key)
    }

    pub fn set<T>(&mut self, key: &str, val: T) -> Result<(), PropErr>
        where Properties: GetProp<T> {
        self.set_prop(key, val)
    }

    pub fn remove(&mut self, key: &str) -> () {
        self.props.remove(key);
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
trait GetProp<T> {
    fn get_prop(&self, key: &str) -> Result<T, PropErr>;
    fn set_prop(&mut self, key: &str, val: T) -> Result<(), PropErr>;
    fn remove_prop(&mut self, key: &str) -> Result<(), PropErr>;
}

macro_rules! make_get_set {
    ($ty:ty, $path:path) => {
        impl GetProp<$ty> for Properties {
            fn get_prop(&self, key: &str) -> Result<$ty, PropErr> {
                if let Some(prop) = self.props.get(key) {
                    if let $path(val) = *prop {
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
        let res = props.set("wobbly", true);
        assert!(res.is_ok());

        let wobbly = props.get::<bool>("wobbly").unwrap();
        assert_eq!(wobbly, true);

        let flimsy = props.get::<bool>("flimsy");
        assert!(flimsy.is_err());

        let res = props.set("wobbly", 123);
        assert!(res.is_err());

        let res = props.set("wobbly", false);
        assert!(res.is_ok());

        // let res = props.set(IsWobbly, false);
        // let res = props.get(IsWobbly).unwrap();
    }

    #[test]
    fn test_remove() {
        let mut props = Properties::new();
        let res = props.set("wobbly", true);
        assert!(res.is_ok());

        props.remove("wobbly");

        let res = props.get::<bool>("wobbly");
        assert!(res.is_err());
    }
}
