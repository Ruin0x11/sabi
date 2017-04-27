use std::fmt;

use point::Point;
use chunk::CHUNK_WIDTH;

// Because a world position and chunk index are different quantities, newtype to
// enforce correct usage
#[derive(Serialize, Deserialize, Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub struct ChunkIndex(pub Point);

impl ChunkIndex {
    pub fn new(x: i32, y: i32) -> Self {
        ChunkIndex(Point::new(x, y))
    }

    pub fn from_world_pos(pos: Point) -> ChunkIndex {
        let conv = |i: i32| {
            if i < 0 {
                ((i + 1) / CHUNK_WIDTH) - 1
            } else {
                i / CHUNK_WIDTH
            }
        };

        ChunkIndex::new(conv(pos.x), conv(pos.y))
    }

}

impl fmt::Display for ChunkIndex {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "({}, {})", self.0.x, self.0.y)
    }
}
