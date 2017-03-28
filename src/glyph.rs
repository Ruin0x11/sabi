use color::{self, Color};

pub struct RenderableGlyph {
    pub ch: char,
    pub color_fg: Color,
    pub color_bg: Color,
}

#[derive(Debug, Copy, Clone)]
pub enum Glyph {
    Player,

    Dood,

    Floor,
    Wall,
    Debug(char),
    DebugDraw,

    None,
}

impl From<Glyph> for RenderableGlyph {
    fn from(glyph: Glyph) -> RenderableGlyph {
        match glyph {
            Glyph::Player => RenderableGlyph { ch: '@',
                                               color_fg: color::RED,
                                               color_bg: color::BLACK },
            Glyph::Dood   => RenderableGlyph { ch: 'p',
                                               color_fg: color::BLUE,
                                               color_bg: color::BLACK },
            Glyph::Floor => RenderableGlyph  { ch: '.',
                                               color_fg: color::WHITE,
                                               color_bg: color::BLACK },
            Glyph::Wall  => RenderableGlyph  { ch: '#',
                                               color_fg: color::WHITE,
                                               color_bg: color::BLACK },
            Glyph::Debug(ch)  => RenderableGlyph  { ch: ch,
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

