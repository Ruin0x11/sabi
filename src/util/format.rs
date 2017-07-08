use std::io;
use std::io::prelude::*;

use calx_ecs::Entity;

use util::grammar::{self, VerbTense};
use logic::entity::*;
use world::World;

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
                // literal '<' is "<<"
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
                let text = expand_format_specifier(c, entity, world);
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

fn expand_format_specifier(c: char, entity: Entity, world: &World) -> String {
    match c {
        'u' => entity.verb_person(world).pronoun().to_string(),
        'U' => entity.name(world),
        'r' => entity.verb_person(world).possessive().to_string(),
        'R' => {
            let name = entity.name(world);
            grammar::make_possessive(&name)
        },
        'A' => entity.verb_person(world).accusative().to_string(),
        '%' => "%".to_string(),
        _ => "".to_string(),
    }
}

/// Formats a string with some properties of the provided entity. Format specifiers and tagged
/// verbs can be used.
///
/// To conjugate a verb, surround the infinitive with angle brackets ("<>").
///
/// For easier use with `format!`, see `format_mes!`
///
/// Available format specifiers:
/// - `%u`: Pronoun, "I/it/they"
/// - `%U`: Full name, "the putit"
/// - `%r`: Possessive, "my/its/their"
/// - `%R`: Named possessive, "the putit's"
/// - `%A`: Accusative, "me/it/them"
/// - `%%`: Literal '%'
/// - `<<`: Literal '<'
///
/// ```no_run
/// format_message("%U <hit> the brick wall. It hurts %A.", entity, world);
/// ```
pub fn format_message(s: &str, entity: Entity, world: &World) -> String {
    let mut writer = Vec::with_capacity(s.len());
    format_message_internal(s, &mut writer, entity, world).unwrap();
    let mes = String::from_utf8(writer).unwrap();
    grammar::capitalize(&mes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use testing::*;
    use world::traits::*;
    use ecs;
    use ecs::components::Name;
    use point::POINT_ZERO;

    #[test]
    fn test_format() {
        let mut context = test_context();
        let world = &mut context.state.world;
        let player = world.player().unwrap();
        let e = world.spawn(&ecs::prefab::mob("putit", 1000000, "putit").c(Name::new("putit".to_string())), POINT_ZERO);

        assert_eq!(&format_message("%u <kill> it.", player, world), "You kill it.");
        assert_eq!(&format_message("%u <target> you.", e, world), "It targets you.");
        // assert_eq!(&format_message("%U <evaporate>!", e, world), "The putit evaporates!");

        assert_eq!(&format_message("%R dreams sound brightly.", player, world),
                   "Your dreams sound brightly.");
        // assert_eq!(&format_message("%R dreams sound brightly.", e, world),
                   "The putit's dreams sound brightly.");
        assert_eq!(&format_message("%r parameters:", e, world), "Its parameters:");

        assert_eq!(&format_message("<<>>", player, world), "<>>");
        assert_eq!(&format_message("%%", player, world), "%");
    }
}
