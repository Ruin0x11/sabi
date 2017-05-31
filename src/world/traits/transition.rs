// FIXME: Move these appropriately.
// FIXME: Refactor.

pub type TransitionResult<T> = Result<T, ()>;

/// A trait for transitioning between game worlds.
pub trait Transition<T> {
    fn map_id(&self) -> u32;
    fn set_map_id(&mut self, id: u32);
    fn get_transition_data(&mut self) -> TransitionResult<T>;
    fn inject_transition_data(&mut self, data: T) -> TransitionResult<()>;
}
