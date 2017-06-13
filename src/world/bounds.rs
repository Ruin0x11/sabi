use terrain::traits::BoundedTerrain;

use chunk::*;
use point::{Point, POINT_ZERO};


#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum Bounds {
    Unbounded,
    Bounded(i32, i32),
}

impl BoundedTerrain<Point, ChunkIndex> for Bounds {
    fn in_bounds(&self, pos: &Point) -> bool {
        match *self {
            Bounds::Unbounded => true,
            Bounds::Bounded(w, h) => *pos >= POINT_ZERO && *pos < Point::new(w, h)
        }
    }

    fn index_in_bounds(&self, index: &ChunkIndex) -> bool {
        let pos = Point::from(*index);
        self.in_bounds(&pos)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::i32;
    use chunk::CHUNK_WIDTH;

    #[test]
    fn test_unbounded() {
        let bounds = Bounds::Unbounded;

        assert!(bounds.in_bounds(&Point::new(i32::MAX, i32::MAX)));
        assert!(bounds.in_bounds(&Point::new(i32::MIN, i32::MIN)));
    }

    #[test]
    fn test_bounded() {
        let bounds = Bounds::Bounded(64, 32);

        assert!(bounds.in_bounds(&Point::new(0, 0)));
        assert!(bounds.in_bounds(&Point::new(32, 16)));
        assert!(bounds.in_bounds(&Point::new(63, 31)));

        assert!(!bounds.in_bounds(&Point::new(64, 31)));
        assert!(!bounds.in_bounds(&Point::new(63, 32)));

        assert!(!bounds.in_bounds(&Point::new(-1, 0)));
        assert!(!bounds.in_bounds(&Point::new(0, -1)));

        assert!(!bounds.in_bounds(&Point::new(i32::MAX, i32::MAX)));
        assert!(!bounds.in_bounds(&Point::new(i32::MIN, i32::MIN)));
    }

    #[test]
    fn test_bounded_index() {
        let bounds = Bounds::Bounded(3 * CHUNK_WIDTH / 2, 2*CHUNK_WIDTH);

        assert!(bounds.index_in_bounds(&ChunkIndex::new(0, 0)));
        assert!(bounds.index_in_bounds(&ChunkIndex::new(1, 1)));

        assert!(!bounds.index_in_bounds(&ChunkIndex::new(1, 2)));
        assert!(!bounds.index_in_bounds(&ChunkIndex::new(2, 1)));

        assert!(!bounds.index_in_bounds(&ChunkIndex::new(-1, 0)));
        assert!(!bounds.index_in_bounds(&ChunkIndex::new(0, -1)));
    }

}
