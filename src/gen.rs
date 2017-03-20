use tile::*;
use point::Point;
use world::*;

struct Rect {
    pos: Point,
    min: Point,
    bottom_right: Point,
    done: bool,
}

impl Iterator for Rect {
    type Item = Point;
    
    fn next(&mut self) -> Option<Point> {
        if self.done {
            return None;
        }

        let current_point = self.pos;

        if self.pos == self.bottom_right {
            self.done = true;
        }

        if current_point.y == self.min.y ||
            current_point.y == self.bottom_right.y {
                self.pos.x += 1;
                if self.pos.x > self.bottom_right.x {
                    self.pos.y += 1;
                    self.pos.x = self.min.x;
                }
            } else {
                if self.pos.x == self.min.x {
                    self.pos.x = self.bottom_right.x;
                } else {
                    self.pos.y += 1;
                    self.pos.x = self.min.x;
                }
            }
        
        Some(current_point)
    }
}

impl Rect {
    pub fn new<P: Into<Point>>(top_left: P, bottom_right: P) -> Self {
        let point = top_left.into();
        Rect {
            pos: point.clone(),
            min: point.clone(),
            bottom_right: bottom_right.into(),
            done: false,
        }
    }
}

impl World {
    fn debug_cell(&self, pos: WorldPosition) {
        if let Some(cell) = self.cell(pos) {
            debug!(self.logger, "Tile before: {:?}", cell.tile);
        }
    }

    pub fn set_tile(&mut self, pos: WorldPosition, tile: Tile) {
        self.debug_cell(pos);
        if let Some(cell_mut) = self.cell_mut(pos) {
            cell_mut.tile = tile.clone();
        }
    }

    pub fn set_tile_feature(&mut self, pos: WorldPosition, feature: Option<TileFeature>) {
        if let Some(cell_mut) = self.cell_mut(pos) {
            cell_mut.tile.feature = feature;
        }
    }

    pub fn draw_rect(&mut self,
                     top_left: WorldPosition,
                     bottom_right: WorldPosition,
                     tile: Tile) {
        let rect = Rect::new(top_left, bottom_right);
        for pos in rect {
            debug!(self.logger, "Position: {}", pos);
            self.set_tile(pos, tile);
        }
    }
}
