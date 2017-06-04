use std::collections::VecDeque;

use graphics::Color;
use point::Point;

pub struct Marks {
    marks: VecDeque<Mark>
}

pub struct Mark {
    pub pos: Point,
    pub color: Color,
}

impl Marks {
    pub fn new() -> Self {
        Marks {
            marks: VecDeque::new(),
        }
    }

    pub fn add(&mut self, pos: Point, color: Color) {
        let mark = Mark { pos: pos, color: color };
        self.marks.push_back(mark);
    }

    pub fn clear(&mut self) {
        self.marks.clear();
    }

    pub fn iter<'a>(&'a self) -> impl Iterator<Item=&'a Mark> {
        self.marks.iter()
    }
}
