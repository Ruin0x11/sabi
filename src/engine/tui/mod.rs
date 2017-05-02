use std::cmp;
use engine::canvas::Canvas;
use point::{Point, POINT_ZERO};

pub enum ViewEvent {
    Key
}

pub enum ViewEventResult {
    Ignored
}

pub struct Printer<'a> {
    canvas: &'a Canvas,
    size: Point,
    offset: Point,
}

impl<'a> Printer<'a> {
    pub fn new(canvas: &'a Canvas, size: Point) -> Self {
        Printer {
            canvas: canvas,
            size: size,
            offset: POINT_ZERO,
        }
    }

    pub fn print(&self, p: Point, text: &str) {
        if p.y >= self.size.y || p.x >= self.size.x {
            return;
        }

        let room = self.size.x - p.x;

        // let prefix_len = prefix(text.graphemes(true), room, "").length;
        // let text = &text[..prefix_len];

        let p = p + self.offset;
        self.canvas.print_str(p.x, p.y, text);
    }

    pub fn print_box(&self, start: Point, size: Point) {
        if size.x < 2 || size.y < 2 {
            return;
        }
        let size = size - (1, 1);

        self.print(start, "┌");
        self.print(start + size.keep_y(), "└");
        self.print_hline(start + (1, 0), size.x - 1, "─");
        self.print_vline(start + (0, 1), size.y - 1, "│");

        self.print(start + size.keep_x(), "┐");
        self.print(start + size, "┘");
        self.print_hline(start + (1, 0) + size.keep_y(), size.x - 1, "─");
        self.print_vline(start + (0, 1) + size.keep_x(), size.y - 1, "│");
    }

    pub fn print_vline(&self, p: Point, len: i32, c: &str) {
        if p.y > self.size.y || p.x > self.size.x {
            return;
        }
        let len = cmp::min(len, self.size.y - p.y);

        let p = p + self.offset;
        for y in 0..len {
            self.canvas.print_str(p.x, p.y + y, c);
        }
    }

    pub fn print_hline(&self, p: Point, len: i32, c: &str) {
        if p.y > self.size.y || p.x > self.size.x {
            return;
        }
        let len = cmp::min(len, self.size.x - p.x) as usize;
        let text: String = ::std::iter::repeat(c).take(len).collect();

        let p = p + self.offset;
        self.canvas.print_str(p.x, p.y, &text);
    }
}

struct Tui<'a> {
    canvas: &'a Canvas,
}

trait View {
    fn draw(&self, canvas: &Canvas);

    fn on_event(&self, event: ViewEvent) -> ViewEventResult {
        ViewEventResult::Ignored
    }

    fn required_size(&mut self) -> Point {
        Point::new(1, 1)
    }
}
