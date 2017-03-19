use color::Color;

pub struct RenderableGlyph {
    pub ch: char,
    pub color_fg: Color,
    pub color_bg: Color,
}

pub enum Glyph {
    Player
}

// DOOD
impl From<Glyph> for RenderableGlyph {
    fn from(glyph: Glyph) -> RenderableGlyph {
        match glyph {
            Glyph::Player => RenderableGlyph { ch: '@',
                                               color_fg: Color { r: 0, g: 0, b: 255 },
                                               color_bg: Color { r: 0, g: 0, b: 0 } }
        }
    }
}
