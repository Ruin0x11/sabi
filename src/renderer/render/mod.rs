pub mod background;
pub mod shadowmap;
pub mod spritemap;
pub mod tilemap;
mod viewport;
mod util;

use std::thread;
use std::time::{Duration, Instant};

use glium;
use glium::glutin;
use glium::Surface;
use glium::backend::Facade;
use glium::index::PrimitiveType;

use graphics::cell::Cell;
use graphics::color::Color;
use renderer::RenderUpdate;
use renderer::ui::*;
use world::World;

use self::background::Background;
use self::shadowmap::ShadowMap;
use self::spritemap::SpriteMap;
use self::tilemap::TileMap;

pub use self::viewport::Viewport;

pub const SCREEN_WIDTH: u32 = 1366;
pub const SCREEN_HEIGHT: u32 = 768;

pub const QUAD_INDICES: [u16; 6] = [0, 1, 2, 1, 3, 2];
pub const QUAD: [Vertex; 4] = [
    Vertex { position: [0, 1] },
    Vertex { position: [1, 1] },
    Vertex { position: [0, 0] },
    Vertex { position: [1, 0] },
];

pub fn load_program<F: Facade>(display: &F,
                               vert: &str,
                               frag: &str)
                               -> Result<glium::Program, glium::ProgramCreationError> {
    let vertex_shader = ::util::read_string(&format!("data/shaders/{}", vert));
    let fragment_shader = ::util::read_string(&format!("data/shaders/{}", frag));

    glium::Program::from_source(display, &vertex_shader, &fragment_shader, None)
}

pub fn make_quad_buffers<F: Facade>(display: &F)
                                    -> (glium::VertexBuffer<Vertex>, glium::IndexBuffer<u16>) {
    let vertices = glium::VertexBuffer::immutable(display, &QUAD).unwrap();
    let indices =
        glium::IndexBuffer::immutable(display, PrimitiveType::TrianglesList, &QUAD_INDICES)
            .unwrap();
    (vertices, indices)
}

#[derive(Copy, Clone)]
pub struct Vertex {
    pub position: [i32; 2],
}

implement_vertex!(Vertex, position);

pub struct RenderContext {
    backend: glium::Display,
    events_loop: glutin::EventsLoop,
    ui: Ui,

    background: Background,
    spritemap: SpriteMap,
    tilemap: TileMap,
    shadowmap: ShadowMap,

    accumulator: FpsAccumulator,
    pub viewport: Viewport,
}

impl RenderContext {
    pub fn new() -> Self {
        let events_loop = glutin::EventsLoop::new();

        let window = glutin::WindowBuilder::new()
            .with_dimensions(SCREEN_WIDTH, SCREEN_HEIGHT)
            .with_title("sabi");

        let context = glutin::ContextBuilder::new();

        let display = glium::Display::new(window, context, &events_loop).unwrap();

        let scale = display.gl_window().hidpi_factor();

        let viewport = Viewport {
            position: (0, 0),
            size: (SCREEN_WIDTH, SCREEN_HEIGHT),
            scale: scale,
            camera: (0, 0),
        };

        let bg = Background::new(&display);
        let ui = Ui::new(&display, &viewport);
        let tile = TileMap::new(&display);
        let shadow = ShadowMap::new(&display);
        let sprite = SpriteMap::new(&display);

        let accumulator = FpsAccumulator::new();

        RenderContext {
            backend: display,
            events_loop: events_loop,
            ui: ui,

            background: bg,
            shadowmap: shadow,
            spritemap: sprite,
            tilemap: tile,
            accumulator: accumulator,
            viewport: viewport,
        }
    }

    pub fn start_loop<F>(&mut self, mut callback: F)
    where
        F: FnMut(&mut RenderContext, glutin::Event) -> Option<Action>,
    {
        let mut closure = |renderer: &mut RenderContext| -> Action {
            let mut events = Vec::new();
            renderer.poll_events(|event| { events.push(event); });

            for event in events {
                if let Some(a) = callback(renderer, event) {
                    return a;
                }
            }
            Action::Continue
        };

        loop {
            match closure(self) {
                Action::Stop => break,
                Action::Continue => (),
            };

            self.render();
            self.step_frame();
        }
    }

    pub fn step_frame(&mut self) {
        self.accumulator.step_frame();

        thread::sleep(self.accumulator.sleep_time());
    }

    // pub fn update(&mut self, board: &Board) {
    //     self.tilemap.update(board);
    // }

    pub fn reload_shaders(&mut self) {
        self.background.reload_shaders(&self.backend);
        self.spritemap.reload_shaders(&self.backend);
        self.shadowmap.reload_shaders(&self.backend);
    }

    pub fn update(&mut self, world: &World) {
        self.tilemap.update(world, &self.viewport);
        self.spritemap.update(world, &self.viewport);
        self.shadowmap.update(world, &self.viewport);
        self.ui.update(world, &self.viewport);
    }

    pub fn render(&mut self) {
        let mut target = self.backend.draw();
        target.clear_color_and_depth((0.0, 0.0, 0.0, 0.0), 1.0);

        let millis = self.accumulator.millis_since_start();

        self.background
            .render(&self.backend, &mut target, &self.viewport, millis);

        self.tilemap.redraw(&self.backend, millis);
        self.tilemap
            .render(&self.backend, &mut target, &self.viewport, millis);

        self.spritemap.redraw(&self.backend, millis);
        self.spritemap
            .render(&self.backend, &mut target, &self.viewport, millis);

        self.shadowmap.redraw(&self.backend, millis);
        self.shadowmap
            .render(&self.backend, &mut target, &self.viewport, millis);

        self.ui
            .render(&self.backend, &mut target, &self.viewport, millis);

        target.finish().unwrap();
    }

    pub fn set_viewport(&mut self, w: u32, h: u32) {
        let scale = self.backend.gl_window().hidpi_factor();
        self.viewport = Viewport {
            position: (0, 0),
            size: (w, h),
            scale: scale,
            camera: self.viewport.camera,
        };

        self.ui = Ui::new(&self.backend, &self.viewport);
    }

    pub fn poll_events<F>(&mut self, callback: F)
    where
        F: FnMut(glutin::Event),
    {
        self.events_loop.poll_events(callback)
    }

    // pub fn update_ui(&mut self, event: &glutin::Event) -> bool {
    //     if self.ui.is_active() {
    //         self.ui.on_event(event.clone());
    //         self.ui.update();
    //         return true;
    //     } else {
    //         self.ui.update();
    //         return false;
    //     }
    // }

    pub fn query<R, T: 'static + UiQuery<QueryResult = R>>(&mut self, layer: &mut T) -> Option<R> {
        loop {
            let mut result = None;
            let mut found = false;
            let mut update = false;
            self.events_loop.poll_events(|event| match event {
                                             glutin::Event::WindowEvent { event, .. } => {
                                                 match layer.on_event(event) {
                                                     EventResult::Done => {
                                                         result = layer.result();
                                                         found = true;
                                                     },
                                                     EventResult::Canceled => {
                                                         result = None;
                                                         found = true;
                                                     },
                                                     _ => update = true,
                                                 }
                                             },
                                             _ => update = true,
                                         });

            if found {
                return result;
            }

            // Don't redraw every frame if it can be helped
            if update {
                self.ui.render_all();
                self.ui.draw_layer(layer);
            }

            self.render();
            self.accumulator.step_frame();
        }

        None
    }

    pub fn cell_to_color(&self, tile: &Cell) -> Color {
        let tile_glyph = tile.glyph();
        self.tilemap.get_tile(tile_glyph).color
    }
}

pub trait Renderable {
    fn render<F, S>(&self, display: &F, target: &mut S, viewport: &Viewport, time: u64)
    where
        F: glium::backend::Facade,
        S: glium::Surface;
}

pub enum Action {
    Stop,
    Continue,
}

pub struct FpsAccumulator {
    start: Instant,
    frame_count: u32,
    last_time: u64,
    accumulator: Duration,
    previous_clock: Instant,
}

impl FpsAccumulator {
    pub fn new() -> Self {
        FpsAccumulator {
            start: Instant::now(),
            frame_count: 0,
            last_time: 0,
            accumulator: Duration::new(0, 0),
            previous_clock: Instant::now(),
        }
    }

    pub fn step_frame(&mut self) {
        let now = Instant::now();
        self.accumulator += now - self.previous_clock;
        self.previous_clock = now;

        let fixed_time_stamp = Duration::new(0, 16666667);
        while self.accumulator >= fixed_time_stamp {
            self.accumulator -= fixed_time_stamp;
        }

        let millis = ::util::get_duration_millis(&Instant::now().duration_since(self.start));

        if millis - self.last_time >= 1000 {
            let ms_per_frame = 1000.0 / self.frame_count as f32;
            println!("{} ms/frame | {} fps", ms_per_frame, 1000.0 / ms_per_frame);
            self.frame_count = 0;
            self.last_time += 1000;
        }

        self.frame_count += 1;
    }

    pub fn sleep_time(&self) -> Duration {
        Duration::new(0, 16666667) - self.accumulator
    }

    pub fn millis_since_start(&self) -> u64 {
        let duration = Instant::now().duration_since(self.start);
        ::util::get_duration_millis(&duration)
    }
}
