use color::{self, Color};

pub struct RenderableGlyph {
    pub ch: char,
    pub color_fg: Color,
    pub color_bg: Color,
}

pub enum Glyph {
    Player
}

impl From<Glyph> for RenderableGlyph {
    fn from(glyph: Glyph) -> RenderableGlyph {
        match glyph {
            Glyph::Player => RenderableGlyph { ch: '@',
                                               color_fg: color::WHITE,
                                               color_bg: color::BLACK }
        }
    }
}

