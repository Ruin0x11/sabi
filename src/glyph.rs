use std::collections::HashMap;

use color::{self, Color};
use toml::Value;
use util::toml::*;

fn char_from_string(s: String) -> char {
    s.chars().nth(0).unwrap()
}

fn make_glyph_table() -> HashMap<Glyph, RenderGlyph> {
    let mut glyphs = HashMap::new();
    let val = toml_value_from_file("./data/glyphs.toml");

    let glyph_names = match val {
        Value::Table(ref table) => table.keys().cloned(),
        _           => panic!("Glyph table wasn't a table."),
    };

    for name in glyph_names {
        let glyph_name = match name.parse::<Glyph>() {
            Ok(n)   => n,
            Err(..) => panic!("Glyph name \"{}\" not found", name)
        };

        let make_color = |rgb: Vec<u8>| Color::new(rgb[0], rgb[1], rgb[2]);

        let ch_string: String = expect_toml_value(&val, &name, "ch");
        let color = make_color(expect_toml_value(&val, &name, "color"));
        let color_bg = match get_toml_value(&val, &name, "color_bg") {
            Some(c) => make_color(c),
            None    => color::BLACK,
        };

        let glyph = RenderGlyph {
            ch: char_from_string(ch_string),
            color_fg: color,
            color_bg: color_bg,
        };
        glyphs.insert(glyph_name, glyph);
    }
    glyphs
}

pub fn lookup_ascii<'a>(glyph: Glyph) -> &'a RenderGlyph {
    match GLYPH_TABLE.get(&glyph) {
        Some(rg) => rg,
        None     => panic!("Glyph name {:?} is specified, but no corresponding glyph was found!", glyph)
    }
}

lazy_static! {
    static ref GLYPH_TABLE: HashMap<Glyph, RenderGlyph> = make_glyph_table();
}

pub struct RenderGlyph {
    pub ch: char,
    pub color_fg: Color,
    pub color_bg: Color,


    // In the future, could hold information on how to draw the character in
    // graphically based rendering tagets.
    //pub graphical_data: Option<char>,
}

macro_attr!(
    #[derive(Hash, Eq, PartialEq, Debug, Copy, Clone, EnumFromStr!, Serialize, Deserialize)]
    pub enum Glyph {
        Player,

        Putit,
        Prinny,

        Floor,
        Wall,
        DebugDraw,

        Item,

        None,
    });
