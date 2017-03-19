use color::Color;

pub struct RenderableGlyph {
    pub ch: char,
    pub color: Color,
}

pub enum Glyph {
    Player
}

// DOOD
impl From<Glyph> for RenderableGlyph {
    fn from(glyph: Glyph) -> RenderableGlyph {
        match glyph {
            Player => RenderableGlyph { ch: '@', color: Color { r: 0, g: 0, b: 255 }}
        }
    }
}
