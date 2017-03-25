/// A message that can be broadcast to an area of a map. After actions are run,
/// things inside the range that need to respond to the message can do so.
struct Message {
    iter: Box<Iterator>,
    kind: MessageKind,
}

enum MessageKind {
    SayName,
    SayThing(String),
}
