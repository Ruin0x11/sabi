use std::cell::RefCell;

use keys::Key;
use glyph::Glyph;
use point::Point;
use ui::*;

// Ok, this is getting tiring attempting to come up with a solution when I can
// just use a global static canvas and be done with it. Here are the kinds of
// problems I ran into trying not to use a global canvas:
// 1. If I need to ask the player in the middle of a command function for a
// yes/no query, there's no way to do it without globals short of passing around
// the canvas to every possible AI action routine.
// 2. With a rendering backend that doesn't block on input and always runs at a
// set framerate, there would have to be some condition earlier in the rendering
// logic if there is some kind of query the player needs to respond to. But then
// execution has to branch at the point where the query was asked, meaning a
// whole bunch of context would be saved because everything is single-threaded.
// For whatever it is I'm trying to do, it seems like just blocking at that
// point is far, far simpler than handling player input earlier and then
// breaking up every relevant function between however many cases there are for
// the player to choose from.
// 3. Every game I've looked at in attempts to see how this problem is dodged
// always just ends up using a blocking global canvas object, or doesn't have a
// player query system at all.
//
// I'm sure this isn't good enough of a justifcation. I don't particularly know
// what I'm doing. But this solves the problem at the expense of blocking at the
// relevant places, which is what many other games do anyway. And the graphics
// are only secondary.
thread_local!(static CANVAS: RefCell<Canvas> = RefCell::new(get_canvas().unwrap()));

pub fn with<A, F>(mut f: F) -> A
    where F: FnMut(&Canvas) -> A {
    CANVAS.with(|w| f(& *w.borrow()))
}

pub fn with_mut<A, F>(mut f: F) -> A
    where F: FnMut(&mut Canvas) -> A {
    CANVAS.with(|w| f(&mut *w.borrow_mut()))
}

pub fn width() -> i32 {
    with(|c| c.width())
}

pub fn height() -> i32 {
    with(|c| c.height())
}

pub fn window_closed() -> bool {
    with(|c| c.window_closed())
}

pub fn get_input() -> Vec<Key> {
    with(|c| c.get_input())
}

pub fn close_window() {
    with_mut(|c| c.close_window());
}


pub fn present() {
    with(|c| c.present())
}

pub fn clear() {
    with(|c| c.clear());
}

/// All rendering targets must follow this API.
pub trait Canvas_ {
    // low-level
    fn width(&self) -> i32;
    fn height(&self) -> i32;
    fn print_info(&self);
    fn clear(&self);
    fn present(&self);
    fn close_window(&mut self);
    fn window_closed(&self) -> bool;

    fn set_camera(&mut self, x: i32, y: i32);
    fn translate_pos(&self, world_x: i32, world_y: i32) -> (i32, i32);

    fn print_str(&self, x: i32, y: i32, s: &str);

    // NOTE: The intention is that this may or may not block depending on the
    // backend, but that might not be a good idea...
    fn get_input(&self) -> Vec<Key>;

    fn draw_window(&self, kind: WindowKind);

    // high-level

    // NOTE: This is a bit high-level, but backends like OpenGL will have no
    // concept of foreground/background colors.
    fn print_glyph(&self, x: i32, y: i32, glyph: Glyph);

    fn print_messages(&self);
    fn update_message_buffer(&mut self, messages: Vec<String>);

    fn point_inside_canvas(&self, pos: Point) -> bool {
        let w = self.width();
        let h = self.height();

        pos.x >= 0 && pos.y >= 0
            && pos.x < w && pos.y < h
    }
}

pub type Canvas = Box<Canvas_>;

#[cfg(feature = "with-opengl")]
fn get_canvas_opengl() -> Option<Canvas> {
    use engine::glium;
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
    use engine::tcod;
    let canvas = Box::new(tcod::TcodCanvas::new(Point::new(80, 25),
                                                "sabi"));
    Some(canvas)
}

#[cfg(not(feature = "with-tcod"))]
fn get_canvas_tcod() -> Option<Canvas> {
    None
}

// #[cfg(feature = "with-rustbox")]
fn get_canvas_rustbox() -> Option<Canvas> {
    use engine::rustbox;
    let canvas = Box::new(rustbox::RustboxCanvas::new(Point::new(80, 25),
                                                      "sabi"));
    Some(canvas)
}

// #[cfg(not(feature = "with-rustbox"))]
// fn get_canvas_rustbox() -> Option<Canvas> {
//     None
// }

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

pub trait Render {
    fn render(&self, canvas: &Canvas);
}
