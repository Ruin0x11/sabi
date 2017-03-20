use glyph::Glyph;

#[derive(Copy, Clone)]
pub enum TileType {
    Wall,
    Floor,
    Air,
    Water,
    Lava
}

#[derive(Copy, Clone)]
pub enum TileFeature {
    Door(bool),
    StairsUp,
    StairsDown,
}

#[derive(Copy, Clone)]
pub struct Tile {
    pub type_: TileType,
    pub glyph: Glyph,
    pub feature: Option<TileFeature>,
}
