use color::{self, Color};

pub struct RenderableGlyph {
    pub ch: char,
    pub color_fg: Color,
    pub color_bg: Color,
}

macro_attr!(
#[derive(Eq, PartialEq, Debug, Copy, Clone, EnumFromStr!)]
pub enum Glyph {
    Player,

    Prinny,

    Floor,
    Wall,
    DebugDraw,

    None,
});

impl From<Glyph> for RenderableGlyph {
    fn from(glyph: Glyph) -> RenderableGlyph {
        match glyph {
            Glyph::Player => RenderableGlyph { ch: '@',
                                               color_fg: color::RED,
                                               color_bg: color::BLACK },
            Glyph::Prinny => RenderableGlyph { ch: 'p',
                                               color_fg: color::BLUE,
                                               color_bg: color::BLACK },
            Glyph::Floor => RenderableGlyph  { ch: '.',
                                               color_fg: color::WHITE,
                                               color_bg: color::BLACK },
            Glyph::Wall  => RenderableGlyph  { ch: '#',
                                               color_fg: color::WHITE,
                                               color_bg: color::BLACK },
            Glyph::DebugDraw  => RenderableGlyph  { ch: 'X',
                                                    color_fg: color::WHITE,
                                                    color_bg: color::RED },
            Glyph::None  => RenderableGlyph  { ch: ' ',
                                               color_fg: color::BLACK,
                                               color_bg: color::BLACK },
        }
    }
}

