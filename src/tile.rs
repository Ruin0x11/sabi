use glyph::Glyph;

#[derive(Debug, Copy, Clone)]
pub enum TileType {
    Wall,
    Floor,
    Air,
    Water,
    Lava
}

#[derive(Debug, Copy, Clone)]
pub enum TileFeature {
    Door(bool),
    StairsUp,
    StairsDown,
}

#[derive(Debug, Copy, Clone)]
pub struct Tile {
    pub type_: TileType,
    pub glyph: Glyph,
    pub feature: Option<TileFeature>,
}
