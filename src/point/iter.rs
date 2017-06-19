use point::Point;

#[derive(Clone)]
pub struct CircleIter {
    pos: Point,
    center: Point,
    radius: i32,
    initial_x: i32,
    max: Point,
}

impl CircleIter {
    pub fn new<P: Into<Point>>(center: P, radius: i32) -> Self {
        let center = center.into();
        CircleIter {
            pos: center - (radius, radius),
            center: center,
            radius: radius,
            initial_x: center.x - radius,
            max: center + (radius, radius),
        }
    }
}

impl Iterator for CircleIter {
    type Item = Point;

    fn next(&mut self) -> Option<Point> {
        loop {
            if (self.pos.y > self.max.y) || (self.pos.x > self.max.x) {
                return None;
            }
            let current_point = self.pos;
            self.pos.x += 1;
            if self.pos.x > self.max.x {
                self.pos.x = self.initial_x;
                self.pos.y += 1;
            }
            if self.center.distance(current_point) < self.radius as f32 {
                return Some(current_point);
            } else {
                // Keep looping for another point
            }
        }
    }
}

/// A square area defined by its "half_side" or radius.
/// A half_side of 0 means no points. Radius of 1 means the centre point.
/// Radius of 2 means a square of 9 points, and so on.
#[derive(Clone)]
pub struct SquareIter {
    pos: Point,
    min_x: i32,
    radius: i32,
    max: Point,
}

impl SquareIter {
    pub fn new<P: Into<Point>>(center: P, radius: i32) -> Self {
        let center = center.into();
        let half_side = radius;
        SquareIter {
            radius: radius,
            pos: center - (half_side, half_side),
            min_x: center.x - half_side,
            max: center + (half_side, half_side),
        }
    }
}

impl Iterator for SquareIter {
    type Item = Point;

    fn next(&mut self) -> Option<Point> {
        if self.pos.y > self.max.y {
            return None;
        }

        if self.radius == 0 {
            let res = self.pos;
            self.pos.y += 1;
            return Some(res);
        }

        let current_point = self.pos;
        self.pos.x += 1;
        if self.pos.x > self.max.x {
            self.pos.y += 1;
            self.pos.x = self.min_x;
        }
        Some(current_point)
    }
}

#[derive(Clone)]
pub struct RectangleIter {
    pos: Point,
    top_left: Point,
    bottom_right: Point,
    done: bool,
}

impl RectangleIter {
    pub fn new<P: Into<Point>>(top_left: P, size: P) -> Self {
        let start = top_left.into();
        RectangleIter {
            pos: start,
            top_left: start,
            bottom_right: start + size.into(),
            done: false,
        }
    }
}

impl Iterator for RectangleIter {
    type Item = Point;

    fn next(&mut self) -> Option<Point> {
        if self.done {
            return None;
        }

        let current_point = self.pos;

        if self.pos == self.bottom_right {
            self.done = true;
        }

        self.pos.x += 1;
        if self.pos.x > self.bottom_right.x {
            self.pos.y += 1;
            self.pos.x = self.top_left.x;
        }

        Some(current_point)
    }
}

#[derive(Clone)]
pub struct BorderIter {
    pos: Point,
    top_left: Point,
    bottom_right: Point,
    done: bool,
}

impl Iterator for BorderIter {
    type Item = Point;

    fn next(&mut self) -> Option<Point> {
        if self.done {
            return None;
        }

        let current_point = self.pos;

        if self.pos == self.bottom_right {
            self.done = true;
        }

        if current_point.y == self.top_left.y || current_point.y == self.bottom_right.y {
            self.pos.x += 1;
            if self.pos.x > self.bottom_right.x {
                self.pos.y += 1;
                self.pos.x = self.top_left.x;
            }
        } else if self.pos.x == self.top_left.x {
            self.pos.x = self.bottom_right.x;
        } else {
            self.pos.y += 1;
            self.pos.x = self.top_left.x;
        }

        Some(current_point)
    }
}

impl BorderIter {
    pub fn new<P: Into<Point>>(top_left: P, bottom_right: P) -> Self {
        let start = top_left.into();
        BorderIter {
            pos: start,
            top_left: start,
            bottom_right: bottom_right.into(),
            done: false,
        }
    }
}

#[derive(Clone)]
pub struct LineIter {
    pos: Point,
    dx: i32,
    dy: i32,
    x1: i32,
    diff: i32,
    octant: Octant,
}

#[derive(Clone)]
struct Octant(u8);

impl Octant {
    fn from_points(start: Point, end: Point) -> Octant {
        let mut dx = end.x - start.x;
        let mut dy = end.y - start.y;

        let mut octant = 0;

        if dy < 0 {
            dx = -dx;
            dy = -dy;
            octant += 4;
        }

        if dx < 0 {
            let tmp = dx;
            dx = dy;
            dy = -tmp;
            octant += 2
        }

        if dx < dy {
            octant += 1
        }

        Octant(octant)
    }

    fn to_octant0(&self, p: Point) -> Point {
        let point = match self.0 {
            0 => (p.x, p.y),
            1 => (p.y, p.x),
            2 => (p.y, -p.x),
            3 => (-p.x, p.y),
            4 => (-p.x, -p.y),
            5 => (-p.y, -p.x),
            6 => (-p.y, p.x),
            7 => (p.x, -p.y),
            _ => unreachable!(),
        };
        point.into()
    }

    fn from_octant0(&self, p: Point) -> Point {
        let point = match self.0 {
            0 => (p.x, p.y),
            1 => (p.y, p.x),
            2 => (-p.y, p.x),
            3 => (-p.x, p.y),
            4 => (-p.x, -p.y),
            5 => (-p.y, -p.x),
            6 => (p.y, -p.x),
            7 => (p.x, -p.y),
            _ => unreachable!(),
        };
        point.into()
    }
}

impl LineIter {
    pub fn new<P: Into<Point>>(start: P, end: P) -> LineIter {
        let start = start.into();
        let end = end.into();

        let octant = Octant::from_points(start, end);

        let start = octant.to_octant0(start);
        let end = octant.to_octant0(end);

        let dx = end.x - start.x;
        let dy = end.y - start.y;

        LineIter {
            pos: start,
            dx: dx,
            dy: dy,
            x1: end.x,
            diff: dy - dx,
            octant: octant,
        }
    }
}

impl Iterator for LineIter {
    type Item = Point;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos.x >= self.x1 {
            return None;
        }

        let p = self.pos;

        if self.diff >= 0 {
            self.pos.y += 1;
            self.diff -= self.dx;
        }

        self.diff += self.dy;

        // loop inc
        self.pos.x += 1;

        Some(self.octant.from_octant0(p))
    }
}

#[cfg(test)]
mod test {
    use std::iter::FromIterator;
    use point::Point;
    use super::*;

    #[test]
    fn test_rectangle() {
        let actual: Vec<Point> = FromIterator::from_iter(RectangleIter::new((-1, -1), (2, 3)));
        let expected = [
            (-1, -1),
            (0, -1),
            (1, -1),
            (-1, 0),
            (0, 0),
            (1, 0),
            (-1, 1),
            (0, 1),
            (1, 1),
            (-1, 2),
            (0, 2),
            (1, 2),
        ];
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_border() {
        let actual: Vec<Point> = FromIterator::from_iter(BorderIter::new((-1, -1), (1, 2)));
        let expected = [
            (-1, -1),
            (0, -1),
            (1, -1),
            (-1, 0),
            (1, 0),
            (-1, 1),
            (1, 1),
            (-1, 2),
            (0, 2),
            (1, 2),
        ];
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_points_within_radius_of_zero() {
        let actual: Vec<Point> = FromIterator::from_iter(SquareIter::new((3, 3), 0));
        assert_eq!(actual, [(3, 3)]);
    }

    #[test]
    fn test_points_within_radius_of_one() {
        let actual: Vec<Point> = FromIterator::from_iter(SquareIter::new((0, 0), 1));
        let expected = [
            (-1, -1),
            (0, -1),
            (1, -1),
            (-1, 0),
            (0, 0),
            (1, 0),
            (-1, 1),
            (0, 1),
            (1, 1),
        ];
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_points_within_radius_of_five() {
        let actual: Vec<Point> = FromIterator::from_iter(SquareIter::new((0, 0), 5));

        let mut expected = Vec::new();
        for y in -5..6 {
            for x in -5..6 {
                expected.push(Point { x: x, y: y });
            }
        }
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_wp_example() {
        let bi = LineIter::new(Point::new(0, 1), Point::new(6, 4));
        let res: Vec<_> = bi.collect();

        assert_eq!(res, [(0, 1), (1, 1), (2, 2), (3, 2), (4, 3), (5, 3)])
    }

    #[test]
    fn test_inverse_wp() {
        let bi = LineIter::new(Point::new(6, 4), Point::new(0, 1));
        let res: Vec<_> = bi.collect();

        assert_eq!(res, [(6, 4), (5, 4), (4, 3), (3, 3), (2, 2), (1, 2)])
    }

    #[test]
    fn test_straight_hline() {
        let bi = LineIter::new(Point::new(2, 3), Point::new(5, 3));
        let res: Vec<_> = bi.collect();

        assert_eq!(res, [(2, 3), (3, 3), (4, 3)]);
    }

    #[test]
    fn test_straight_vline() {
        let bi = LineIter::new(Point::new(2, 3), Point::new(2, 6));
        let res: Vec<_> = bi.collect();

        assert_eq!(res, [(2, 3), (2, 4), (2, 5)]);
    }
}
