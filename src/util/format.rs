use std::io;
use std::io::prelude::*;

use calx_ecs::Entity;

use util::grammar::{self, VerbTense};
use logic::entity::*;
use world::World;

#[cfg(never)]
macro_rules! format_mes {
    ($world:expr, $entity:expr, $format:expr, $( $x:ident=$y:expr ),+) => {
        $(
            let $x = $y;
        )*;

        let unchomped = format!($format, $($x),+);

        let finished = String::new();
        $w.message(finished);
    };
}

#[derive(PartialEq, Eq)]
enum FormatState {
    Normal,
    Formatter,
    TagBegin,
    Tag,
}

use self::FormatState::*;

#[derive(Debug)]
pub struct FormatErr;

impl From<io::Error> for FormatErr {
    fn from(_: io::Error) -> FormatErr {
        FormatErr
    }
}

fn format_message_internal<W: Write>(
    s: &str,
    writer: &mut W,
    entity: Entity,
    world: &World,
) -> Result<(), FormatErr> {
    let mut state: FormatState = Normal;
    let mut buf = String::with_capacity(8);
    for c in s.chars() {
        match state {
            Normal if c == '<' => state = TagBegin,
            Normal if c == '%' => state = Formatter,
            Normal => write!(writer, "{}", c)?,
            TagBegin if c == '<' => {
                state = Normal;
                write!(writer, "{}", c)?;
            },
            TagBegin => {
                state = Tag;
                buf.push(c);
            },
            Tag if c == '>' => {
                state = Normal;
                let person = entity.verb_person(world);
                let conjugated = grammar::conjugate(&buf, person, VerbTense::Present);
                write!(writer, "{}", conjugated)?;
                buf.clear();
            },
            Tag => buf.push(c),
            Formatter => {
                state = Normal;
                let text = match c {
                    'u' => entity.verb_person(world).pronoun().to_string(),
                    'U' => entity.name(world),
                    'R' => {
                        let name = entity.name(world);
                        grammar::make_possessive(&name)
                    },
                    'r' => entity.verb_person(world).possessive().to_string(),
                    'A' => entity.verb_person(world).accusative().to_string(),
                    _ => "".to_string(),
                };

                write!(writer, "{}", text)?;
            },
        }
    }
    if state != Normal {
        Err(FormatErr)
    } else {
        Ok(())
    }
}

pub fn format_message(s: &str, entity: Entity, world: &World) -> String {
    let mut writer = Vec::with_capacity(s.len());
    format_message_internal(s, &mut writer, entity, world).unwrap();
    let mes = String::from_utf8(writer).unwrap();
    grammar::capitalize(&mes)
}

// fn expand_subject(s: &str, name: &str) -> String {}

#[cfg(test)]
mod tests {
    use super::*;
    use testing::*;
    use world::traits::*;
    use ecs;
    use point::POINT_ZERO;

    #[test]
    fn test_format() {
        let mut context = test_context();
        let world = &mut context.state.world;
        let player = world.player().unwrap();
        let e = world.create(ecs::prefab::mob("putit", 1000000, "putit"), POINT_ZERO);

        assert_eq!(&format_message("%u <kill> it.", player, world), "You kill it.");
        assert_eq!(&format_message("%u <target> you.", e, world), "It targets you.");
        assert_eq!(&format_message("%U <evaporate>!", e, world), "The putit evaporates!");
        assert_eq!(&format_message("%R dreams sound brightly.", player, world),
                   "Your dreams sound brightly.");
        assert_eq!(&format_message("%R dreams sound brightly.", e, world),
                   "The putit's dreams sound brightly.");
        assert_eq!(&format_message("%r parameters:", e, world), "Its parameters:");
        assert_eq!(&format_message("<<>>", player, world), "<>>");
    }
}
