use calx_ecs::{ComponentData, Entity};

use ecs::*;

pub trait ComponentQuery<C: Component> {
    /// Gets the component off this entity or panics.
    fn get_or_err(&self, e: Entity) -> &C;

    /// Gets a component off this entity and runs a callback, with a fallback
    /// value if it doesn't exist.
    fn map_or<F, T>(&self, default: T, callback: F, e: Entity) -> T
        where F: FnOnce(&C,) -> T;

    fn map<F, T>(&self, callback: F, e: Entity) -> Option<T>
        where F: FnOnce(&C,) -> T;

    fn has(&self, e: Entity) -> bool;
}

pub trait ComponentMutate<C: Component> {
    fn get_mut_or_err(&mut self, e: Entity) -> &mut C;
    fn map_mut<F, T>(&mut self, callback: F, e: Entity) -> Option<T>
        where F: FnOnce(&mut C,) -> T;
}

impl<C: Component> ComponentQuery<C> for ComponentData<C> {
    fn get_or_err(&self, e: Entity) -> &C {
        self.get(e).unwrap()
    }

    fn map_or<F, T>(&self, default: T, callback: F, e: Entity) -> T
        where F: FnOnce(&C,) -> T {
        self.get(e).map_or(default, callback)
    }

    fn map<F, T>(&self, callback: F, e: Entity) -> Option<T>
        where F: FnOnce(&C,) -> T {
        self.get(e).map(callback)
    }

    fn has(&self, e: Entity) -> bool {
        self.get(e).is_some()
    }
}

impl<C: Component> ComponentMutate<C> for ComponentData<C> {
    fn get_mut_or_err(&mut self, e: Entity) -> &mut C {
        self.get_mut(e).unwrap()
    }

    fn map_mut<F, T>(&mut self, callback: F, e: Entity) -> Option<T>
        where F: FnOnce(&mut C,) -> T {
        self.get_mut(e).map(callback)
    }
}
