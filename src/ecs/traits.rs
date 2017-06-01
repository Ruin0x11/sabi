use calx_ecs::Entity;

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
