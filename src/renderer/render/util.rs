use point::{Direction, Point};

pub(super) fn dir_to_bit(dir: Direction) -> u8 {
    match dir {
        Direction::NE => 0,
        Direction::N  => 1,
        Direction::NW => 2,
        Direction::E  => 3,
        Direction::W  => 4,
        Direction::SE => 5,
        Direction::S  => 6,
        Direction::SW => 7,
    }
}

pub(super) fn get_neighboring_edges<F>(pos: Point, mut is_neighbor: F) -> u8
    where F: FnMut(Point) -> bool {
    let mut res: u8 = 0;
    for dir in Direction::iter8() {
        let new_pos = pos + *dir;
        let same_type = is_neighbor(new_pos);
        if same_type {
            res |= 1 << dir_to_bit(*dir);
        }
    }
    res
}


