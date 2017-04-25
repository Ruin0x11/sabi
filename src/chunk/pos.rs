use std::fmt;
use chunk::CHUNK_WIDTH;

use point::Point;

#[derive(Hash, Eq, PartialEq, Debug, Clone)]
pub struct ChunkPosition(pub Point);

impl From<Point> for ChunkPosition {
    fn from(pos: Point) -> ChunkPosition {
        assert!(pos.x >= 0);
        assert!(pos.y >= 0);
        assert!(pos.x < CHUNK_WIDTH);
        assert!(pos.y < CHUNK_WIDTH);
        ChunkPosition(pos)
    }
}

impl ChunkPosition {
    pub fn from_world(pos: &Point) -> ChunkPosition {
        let conv = |i: i32| {
            let i_new = i % CHUNK_WIDTH;
            if i_new < 0 {
                CHUNK_WIDTH + i_new
            } else {
                i_new
            }
        };
        ChunkPosition(Point::new(conv(pos.x), conv(pos.y)))
    }
}

impl fmt::Display for ChunkPosition {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
