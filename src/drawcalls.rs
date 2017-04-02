use std::cell::RefCell;
use std::collections::VecDeque;

use engine::Canvas;
use glyph::Glyph;

pub struct DrawCalls {
    draw_calls: RefCell<VecDeque<Draw>>
}

impl DrawCalls {
    pub fn new() -> Self {
        DrawCalls {
            draw_calls: RefCell::new(VecDeque::new()),
        }
    }

    pub fn push(&self, draw_call: Draw) {
        self.draw_calls.borrow_mut().push_back(draw_call);
    }

    pub fn clear(&self) {
        self.draw_calls.borrow_mut().clear();
    }

    pub fn draw_all(&self, canvas: &mut Canvas) {
        for draw_call in self.draw_calls.borrow().iter() {
            draw_call.draw(canvas);
        }
    }
}

pub enum Draw {
    Point(i32, i32),
}

pub trait Drawable {
    fn draw(&self, canvas: &mut Canvas);
}

impl Drawable for Draw {
    fn draw(&self, canvas: &mut Canvas) {
        match *self {
            Draw::Point(x, y) => canvas.print_glyph(x, y, Glyph::DebugDraw)
        }
    }
}
