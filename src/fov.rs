use std::cell::RefCell;
use std::collections::{HashSet, hash_set};
use std::fmt::{self, Display};
use std::f32;

use point::*;

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct Line {
    pub from: Point,
    pub to: Point,
}

impl Line {
    pub fn new(from: Point, to: Point) -> Self {
        Line { from: from, to: to }
    }

    pub fn is_clear_cw(&self, pt: &Point) -> bool {
        self.dtheta(pt) > 0.0
    }

    pub fn is_clear_ccw(&self, pt: &Point) -> bool {
        self.dtheta(pt) < 0.0
    }

    fn dtheta(&self, pt: &Point) -> f32 {
        let conv = |a: &Point, b: &Point| {
            let i = (a.y - b.y) as f32;
            let j = (a.x - b.x) as f32;
            i.atan2(j)
        };
        let theta = conv(&self.to, &self.from);
        let other = conv(pt, &self.from);
        let dt = other - theta;
        if dt > -f32::consts::PI {
            dt
        } else {
            (dt + 2.0 * f32::consts::PI)
        }
    }
}

/// Describes a visible area between two lines, along with the obstructions
/// coming from each.
#[derive(Eq, PartialEq, Clone, Debug)]
pub struct Arc {
    pub steep: Line,
    pub shallow: Line,
    pub steep_bumps: Vec<Point>,
    pub shallow_bumps: Vec<Point>,
}

impl Arc {
    pub fn new(steep: Line, shallow: Line) -> Self {
        Arc {
            steep: steep,
            shallow: shallow,
            steep_bumps: Vec::new(),
            shallow_bumps: Vec::new(),
        }
    }

    /// Marks an obstruction coming from the clockwise direction of the arc.
    pub fn add_steep_bump(&mut self, pt: &Point) {
        let steep_bump = Point::new(pt.x + 1, pt.y);
        self.steep_bumps.push(steep_bump);
        self.steep.to = steep_bump;
        for sb in self.shallow_bumps.iter() {
            if self.steep.is_clear_cw(&sb) {
                self.steep.from = sb.clone();
            }
        }
    }

    /// Marks an obstruction coming from the counterclockwise direction of the
    /// arc.
    pub fn add_shallow_bump(&mut self, pt: &Point) {
        let shallow_bump = Point::new(pt.x, pt.y + 1);
        self.shallow_bumps.push(shallow_bump);
        self.shallow.to = shallow_bump;
        for sb in self.steep_bumps.iter() {
            if self.shallow.is_clear_ccw(&sb) {
                self.shallow.from = sb.clone();
            }
        }
    }

    pub fn hits(&self, pt: &Point) -> bool {
        self.steep.is_clear_ccw(&Point::new(pt.x + 1, pt.y)) &&
            self.shallow.is_clear_cw(&Point::new(pt.x, pt.y + 1))
    }

    /// Determines if the wall at the given point blocks the arc.
    pub fn shade(&mut self, pt: &Point) -> Blocking {
        let steep_block = self.steep.is_clear_cw(&Point::new(pt.x, pt.y + 1));
        let shallow_block = self.shallow.is_clear_ccw(&Point::new(pt.x + 1, pt.y));
        if steep_block && shallow_block {
            // The wall is outside the arc, so it isn't visible
            return Blocking::Complete;
        } else if steep_block {
            self.add_steep_bump(pt);
            return Blocking::Partial;
        } else if shallow_block {
            self.add_shallow_bump(pt);
            return Blocking::Partial;
        } else {
            // The wall is between both lines, so make two new arcs to account
            // for the squares it blocks
            let mut a = self.clone();
            let mut b = self.clone();
            a.add_steep_bump(pt);
            b.add_shallow_bump(pt);
            return Blocking::Nothing(a, b)
        }
    }
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum Blocking {
    Complete,
    Partial,
    Nothing(Arc, Arc),
}

/// Represents a light source with a radius covering a quadrant.
pub struct Light {
    arcs: RefCell<Vec<Arc>>,
}

impl Light {
    pub fn new(radius: i32) -> Self {
        assert!(radius >= 0);
        let wide = Arc::new(
            Line::new(Point::new(1, 0), Point::new(0, radius)),
            Line::new(Point::new(0, 1), Point::new(radius, 0)),
        );
        let mut arcs = Vec::new();
        arcs.push(wide);
        Light {
            arcs: RefCell::new(arcs)
        }
    }

    /// Determines if this light contains an arc that hits the given point,
    /// meaning it is visible.
    pub fn hits(&self, pt: &Point) -> Option<usize> {
        for arc in self.arcs.borrow().iter() {
            if arc.hits(pt) {
                let idx = self.arcs.borrow().iter().position(|a| *a == *arc).unwrap();
                return Some(idx);
            }
        }
        None
    }

    /// Checks the blocking status of an arc at a point and adds any
    /// obstructions, then updates this light's list of arcs.
    pub fn shade(&mut self, arc_idx: usize, pt: &Point) -> usize {
        let res = self.arcs.borrow_mut().get_mut(arc_idx).unwrap().shade(pt);
        match res {
            Blocking::Nothing(arc_a, arc_b) => {
                self.arcs.borrow_mut().remove(arc_idx);
                self.arcs.borrow_mut().insert(arc_idx, arc_b);
                self.arcs.borrow_mut().insert(arc_idx, arc_a);
            }
            Blocking::Partial     => (),
            Blocking::Complete    => {self.arcs.borrow_mut().remove(arc_idx); },
        };
        self.arcs.borrow().len()
    }
}

#[derive(Clone, Debug)]
/// Represents a set of points that are visible.
pub struct FieldOfView {
    visible: HashSet<Point>,
}

impl FieldOfView {
    pub fn new() -> Self {
        FieldOfView {
            visible: HashSet::new(),
        }
    }

    /// Updates this field of view using the Precise Permissive Field of View
    /// algorithm.
    pub fn update<F, G>(&mut self, center: &Point, radius: i32, mut in_bounds: F, mut blocked: G)
        where F: FnMut(&Point) -> bool,
              G: FnMut(&Point) -> bool {
        self.visible.insert(center.clone());

        let mut quadrant = |dx, dy| {
            let mut light = Light::new(radius);
            for dr in 1..radius+1 {
                for i in 0..dr+1 {
                    // Translate the world coordinate into the light's
                    // coordinate space.
                    let cell = Point::new(dr - i, i);
                    let idx_opt = light.hits(&cell);

                    // If the cell is unlit, ignore it.
                    if idx_opt.is_none() {
                        continue;
                    }

                    // If it is in bounds, add the lit cell to the visible
                    // cells.
                    let idx = idx_opt.unwrap();
                    let ax = center.x + cell.x * dx;
                    let ay = center.y + cell.y * dy;
                    let next = Point::new(ax, ay);

                    if in_bounds(&next) {
                        self.visible.insert(next);
                    } else {
                        // Position is invalid, so don't try to check the cell
                        // type there.
                        continue;
                    }

                    // If the cell doesn't block light, no shadows need to be
                    // added.
                    if !blocked(&next) {
                        continue;
                    }

                    // Blocking cells cast shadows.
                    let light_source_count = light.shade(idx, &cell);

                    if light_source_count <= 0 {
                        return;
                    }
                }
            }
        };
        quadrant(-1,  1);
        quadrant(1,   1);
        quadrant(-1, -1);
        quadrant(1,  -1);
    }

    pub fn clear(&mut self) {
        self.visible.clear()
    }

    pub fn iter(&self) -> hash_set::Iter<Point> {
        self.visible.iter()
    }

    pub fn is_visible(&self, pt: &Point) -> bool {
        self.visible.contains(pt)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use testbed::make_grid_from_str;
    use std::cell::RefCell;
    use std::iter::FromIterator;

    pub struct Source {
        pos: Point,
        radius: i32,
    }

    #[derive(Eq, PartialEq, Clone, Debug)]
    pub enum Cell {
        Wall,
        Floor,
        Nothing,
    }

    pub struct Board {
        dimensions: Point,
        tiles: Vec<Cell>,
        pub fov: RefCell<FieldOfView>,
        light: Source,
    }

    impl Board {
        pub fn new(x: i32, y: i32, light_pos: Point, light_radius: i32) -> Self {
            let mut tiles = Vec::new();
            for _ in 0..x {
                for _ in 0..y {
                    tiles.push(Cell::Floor);
                }
            }
            Board {
                dimensions: Point::new(x, y),
                tiles: tiles,
                fov: RefCell::new(FieldOfView::new()),
                light: Source {
                    radius: light_radius,
                    pos: light_pos
                }
            }
        }

        pub fn in_bounds(&self, pt: &Point) -> bool {
            *pt >= Point::new(0, 0) && *pt < self.dimensions
        }

        pub fn set(&mut self, pt: &Point, val: Cell) {
            if self.in_bounds(pt) {
                let idx = (pt.y * self.dimensions.x + pt.x) as usize;
                let mut v = self.tiles.get_mut(idx).unwrap();
                *v = val;
            }
        }

        pub fn get(&self, pt: &Point) -> Cell {
            if self.in_bounds(pt) {
                let idx = (pt.y * self.dimensions.x + pt.x) as usize;
                println!("pt: {} dim: {} idx: {}", pt, self.dimensions, idx);
                self.tiles.get(idx).unwrap().clone()
            } else {
                Cell::Nothing
            }
        }

        pub fn update_fov(&mut self) {
            let in_bounds = |pt: &Point| self.in_bounds(pt);
            let blocked = |pt: &Point| self.get(pt).clone() == Cell::Wall;

            self.fov.borrow_mut().update(&self.light.pos, self.light.radius, in_bounds, blocked);
        }

        pub fn get_visible(&self) -> HashSet<Point> {
            HashSet::from_iter(self.fov.borrow().iter().cloned())
        }
    }

    impl Display for Board {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            for i in 0..self.dimensions.x {
                for j in 0..self.dimensions.y {
                    let pos = Point::new(i, j);
                    if pos == self.light.pos {
                        write!(f, "@")?;
                    } else if self.fov.borrow().is_visible(&pos) {
                        write!(f, "X")?;
                    } else {
                        let ch = match self.get(&pos) {
                            Cell::Wall => "#",
                            Cell::Floor => ".",
                            Cell::Nothing => " ",
                        };
                        write!(f, "{}", ch)?;
                    }
                }
                write!(f, "\n")?;
            }
            Ok(())
        }
    }

    fn make_board(text: &str, radius: i32) -> Board {
        let callback = |pt: &Point, c: char, board: &mut Board| {
            if c == '@' {
                board.light.pos = pt.clone();
            }

            let cell_kind = match c {
                '.' => Cell::Floor,
                '@' => Cell::Floor,
                '#' => Cell::Wall,
                _   => unreachable!(),
            };
            board.set(&pt, cell_kind);
        };
        let make = |dim: Point| Board::new(dim.x, dim.y, POINT_ZERO, radius);
        make_grid_from_str(text, make, callback)
    }

    fn test_harness(board: &str, radius: i32, expected_visible: &[(i32, i32)]) {
        let mut board = make_board(board, radius);
        board.update_fov();
        let visible = board.get_visible();
        let expected = HashSet::from_iter(expected_visible.iter().clone()
                                          .map(|&(a, b)| Point::new(a, b)));
        assert_eq!(visible, expected, "\n{}\nExpect: {:?}\nGot:    {:?}",
                   board, expected, visible);
    }

    #[test]
    fn test_all_blocked() {
        test_harness("
.....
.###.
.#@#.
.###.
.....
", 5, &[(1, 1), (1, 2), (1, 3), (2, 1), (2, 2), (2, 3), (3, 1), (3, 2), (3, 3)]);
    }

    #[test]
    fn test_none_blocked() {
        let mut vis = vec![];
        for i in 0..5 {
            for j in 0..5 {
                vis.push((i, j));
            }
        }
        test_harness("
.....
.....
..@..
.....
.....
", 5, &vis);
    }

    #[test]
    fn test_line_blocking() {
        let mut vis = vec![];
        for i in 1..5 {
            for j in 0..5 {
                vis.push((i, j));
            }
        }
        test_harness("
.#...
.#...
.#@..
.#...
.#...
", 5, &vis);
    }

}
