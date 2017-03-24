use glyph::Glyph;

#[derive(Debug, Copy, Clone)]
pub enum TileType {
    Wall,
    Floor,
    Air,
    Water,
    Lava
}

impl Tile {
    pub fn can_see_through(&self) -> bool {
        match self.type_ {
            TileType::Wall => false,
            _              => true,
        }
    }

    pub fn can_pass_through(&self) -> bool {
        match self.type_ {
            TileType::Wall => false,
            _              => true,
        }
    }
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

    // TEMP: Shouldn't go here, but is instead looked up
    pub glyph: Glyph,

    pub feature: Option<TileFeature>,
}
