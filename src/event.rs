use world::WorldIter;
/// An event that can be broadcast to an area of a map. After actions are run,
/// things inside the range that need to respond to the message can do so.
struct Event {
    iter: Box<WorldIter>,
    kind: EventKind,
}

enum EventKind {
    SayName,
    SayThing(String),
}
