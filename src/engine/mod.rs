#[cfg(feature = "with-opengl")]
pub mod glium;

#[cfg(feature = "with-rustbox")]
pub mod rustbox;

#[cfg(feature = "with-tcod")]
pub mod tcod;

use euclid::point::Point2D;

use color::Color;
use keys::Key;
use glyph::Glyph;

pub type Point = Point2D<i32>;

/// All rendering targets must follow this API.
pub trait Canvas_ {
    fn width(&self) -> i32;
    fn height(&self) -> i32;
    fn print_info(&self);
    fn clear(&mut self);
    fn present(&mut self);
    fn close_window(&mut self);
    fn window_closed(&self) -> bool;

    // NOTE: The intention is that this may or may not block depending on the
    // backend, but that might not be a good idea...
    fn get_input(&self) -> Vec<Key>;

    // NOTE: This is a bit high-level, but backends like OpenGL will have no
    // concept of foreground/background colors.
    fn print_glyph(&mut self, x: i32, y: i32, glyph: Glyph);
}

pub type Canvas = Box<Canvas_>;

#[cfg(feature = "with-opengl")]
fn get_canvas_opengl() -> Option<Canvas> {
    let canvas = Box::new(glium::OpenGLCanvas {
        
    });
    Some(canvas)
}

#[cfg(not(feature = "with-opengl"))]
fn get_canvas_opengl() -> Option<Canvas> {
    None
}


#[cfg(feature = "with-tcod")]
fn get_canvas_tcod() -> Option<Canvas> {
    let canvas = Box::new(tcod::TcodCanvas::new(Point::new(80, 25), "some_game"));
    Some(canvas)
}

#[cfg(not(feature = "with-tcod"))]
fn get_canvas_tcod() -> Option<Canvas> {
    None
}

#[cfg(feature = "with-rustbox")]
fn get_canvas_rustbox() -> Option<Canvas> {
    let canvas = Box::new(rustbox::RustboxCanvas::new(Point::new(80, 25), "some_game"));
    Some(canvas)
}

#[cfg(not(feature = "with-rustbox"))]
fn get_canvas_rustbox() -> Option<Canvas> {
    None
}

pub fn get_canvas() -> Option<Canvas> {
    if let Some(canvas) = get_canvas_rustbox() {
        println!("using termbox backend");
        return Some(canvas);
    }
    if let Some(canvas) = get_canvas_tcod() {
        println!("using tcod backend");
        return Some(canvas);
    }
    if let Some(canvas) = get_canvas_opengl() {
        println!("using opengl backend");
        return Some(canvas);
    }

    println!("no graphics backend was compiled in!");
    None
}

// FIXME: Coherence rules prevent blanket impls without macros, so these are
// regular functions instead of inside Canvas_.

pub fn point_inside_canvas(canvas: &Canvas, pos: Point) -> bool {
    let w = canvas.width();
    let h = canvas.height();

    pos.x >= 0 && pos.y >= 0
        && pos.x < w && pos.y < h
}
