use point::Point;

pub struct CircleArea {
    pos: Point,
    center: Point,
    radius: i32,
    initial_x: i32,
    max: Point
}

impl CircleArea {
    pub fn new<P: Into<Point>>(center: P, radius: i32) -> Self {
        let center = center.into();
        CircleArea {
            pos: center - (radius, radius),
            center: center,
            radius: radius,
            initial_x: center.x - radius,
            max: center + (radius, radius),
        }
    }
}

impl Iterator for CircleArea {
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
                return Some(current_point)
            } else {
                // Keep looping for another point
            }
        }
    }
}

pub struct PointArea {
    pos: Point,
    done: bool,
}

impl PointArea {
    pub fn new<P: Into<Point>>(pos: P) -> Self {
        PointArea {
            pos: pos.into(),
            done: false,
        }
    }
}

impl Iterator for PointArea {
    type Item = Point;

    fn next(&mut self) -> Option<Point> {
        if self.done {
            return None
        }

        self.done = true;
        Some(self.pos)
    }
}

/// A square area defined by its "half_side" or radius.
/// A half_side of 0 means no points. Radius of 1 means the centre point.
/// Radius of 2 means a square of 9 points, and so on.
pub struct SquareArea {
    pos: Point,
    min_x: i32,
    max: Point,
    radius: i32,
}

impl SquareArea {
    pub fn new<P: Into<Point>>(center: P, radius: i32) -> Self {
        let center = center.into();
        let half_side = radius - 1;
        SquareArea {
            radius: radius,
            pos: center - (half_side, half_side),
            min_x: center.x - half_side,
            max: center + (half_side, half_side),
        }
    }
}

impl Iterator for SquareArea {
    type Item = Point;

    fn next(&mut self) -> Option<Point> {
        if self.radius == 0 {
            return None
        }

        if self.pos.y > self.max.y {
            return None
        }
        let current_point = self.pos;
        self.pos.x += 1;
        if self.pos.x > self.max.x {
            self.pos.y += 1;
            self.pos.x = self.min_x;
        }
        return Some(current_point)
    }
}

pub struct RectangleArea {
    pos: Point,
    top_left: Point,
    bottom_right: Point,
    done: bool,
}

impl RectangleArea {
    pub fn new<P: Into<Point>>(top_left: P, bottom_right: P) -> Self {
        let start = top_left.into();
        RectangleArea {
            pos: start.clone(),
            top_left: start.clone(),
            bottom_right: bottom_right.into(),
            done: false,
        }
    }
}

impl Iterator for RectangleArea {
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

pub struct BorderArea {
    pos: Point,
    top_left: Point,
    bottom_right: Point,
    done: bool,
}

impl Iterator for BorderArea {
    type Item = Point;

    fn next(&mut self) -> Option<Point> {
        if self.done {
            return None;
        }

        let current_point = self.pos;

        if self.pos == self.bottom_right {
            self.done = true;
        }

        if current_point.y == self.top_left.y ||
            current_point.y == self.bottom_right.y {
                self.pos.x += 1;
                if self.pos.x > self.bottom_right.x {
                    self.pos.y += 1;
                    self.pos.x = self.top_left.x;
                }
            } else {
                if self.pos.x == self.top_left.x {
                    self.pos.x = self.bottom_right.x;
                } else {
                    self.pos.y += 1;
                    self.pos.x = self.top_left.x;
                }
            }

        Some(current_point)
    }
}

impl BorderArea {
    pub fn new<P: Into<Point>>(top_left: P, bottom_right: P) -> Self {
        let start = top_left.into();
        BorderArea {
            pos: start.clone(),
            top_left: start.clone(),
            bottom_right: bottom_right.into(),
            done: false,
        }
    }
}

#[cfg(test)]
mod test {
    use std::iter::FromIterator;
    use std::f32::EPSILON;
    use point::Point;
    use super::*;

    #[test]
    fn test_points_within_radius_of_zero() {
        let actual: Vec<Point> = FromIterator::from_iter(SquareArea::new((3, 3), 0));
        assert_eq!(actual, [(3, 3)]);
    }

    #[test]
    fn test_points_within_radius_of_one() {
        let actual: Vec<Point> = FromIterator::from_iter(SquareArea::new((0, 0), 1));
        let expected = [(-1, -1), (0, -1), (1, -1),
                        (-1,  0), (0,  0), (1,  0),
                        (-1,  1), (0,  1), (1,  1)];
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_points_within_radius_of_five() {
        let actual: Vec<Point> = FromIterator::from_iter(SquareArea::new((0, 0), 5));

        let mut expected = Vec::new();
        for y in -5..6 {
            for x in -5..6 {
                expected.push(Point{x: x, y: y});
            }
        }
        assert_eq!(actual, expected);
    }
}
