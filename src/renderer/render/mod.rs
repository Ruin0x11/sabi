use std::collections::HashSet;
use std::thread;
use std::time::{Duration, Instant};

use glium;
use glium::glutin;
use glium::{DisplayBuild, Surface};
use glium::backend::glutin_backend::GlutinFacade;
use glium::backend::Facade;
use glium::index::PrimitiveType;

use point::{Point, CircleIter};
use renderer::interop::RenderUpdate;
use renderer::ui::*;
use util;
use GameContext;

pub mod background;
pub mod shadowmap;
pub mod spritemap;
pub mod tilemap;
mod viewport;

use self::background::Background;
use self::shadowmap::ShadowMap;
use self::spritemap::SpriteMap;
use self::tilemap::TileMap;

pub use self::viewport::Viewport;

pub const SCREEN_WIDTH: u32 = 1024;
pub const SCREEN_HEIGHT: u32 = 768;

pub const QUAD_INDICES: [u16; 6] = [0, 1, 2, 1, 3, 2];
pub const QUAD: [Vertex; 4] = [
    Vertex { position: [0, 1], },
    Vertex { position: [1, 1], },
    Vertex { position: [0, 0], },
    Vertex { position: [1, 0], },
];

pub fn load_program<F: Facade>(display: &F, vert: &str, frag: &str) -> Result<glium::Program, glium::ProgramCreationError> {
    let vertex_shader = util::read_string(&format!("data/shaders/{}", vert));
    let fragment_shader = util::read_string(&format!("data/shaders/{}", frag));

    glium::Program::from_source(display, &vertex_shader, &fragment_shader, None)
}

pub fn make_quad_buffers<F: Facade>(display: &F) -> (glium::VertexBuffer<Vertex>, glium::IndexBuffer<u16>) {
    let vertices = glium::VertexBuffer::immutable(display, &QUAD).unwrap();
    let indices = glium::IndexBuffer::immutable(display, PrimitiveType::TrianglesList, &QUAD_INDICES).unwrap();
    (vertices, indices)
}

#[derive(Copy, Clone)]
pub struct Vertex {
    pub position: [i32; 2],
}

implement_vertex!(Vertex, position);

pub struct RenderContext {
    backend: GlutinFacade,
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
        let display = glutin::WindowBuilder::new()
            .with_vsync()
            .with_dimensions(SCREEN_WIDTH, SCREEN_HEIGHT)
            .with_title("Tile")
            .build_glium()
            .unwrap();

        let bg = Background::new(&display);
        let ui = Ui::new(&display);
        let tile = TileMap::new(&display);

        let mut vis = HashSet::new();
        for point in CircleIter::new(Point::new(6, 6), 5) {
            vis.insert(point);
        }

        let shadow = ShadowMap::new(&display);

        let sprite = SpriteMap::new(&display);

        let scale = display.get_window().unwrap().hidpi_factor();

        let accumulator = FpsAccumulator::new();

        let viewport = Viewport {
            position: (0, 0),
            size: (SCREEN_WIDTH, SCREEN_HEIGHT),
            scale: scale,
            camera: (0, 0)
        };

        RenderContext {
            backend: display,
            background: bg,
            ui: ui,
            shadowmap: shadow,
            spritemap: sprite,
            tilemap: tile,
            accumulator: accumulator,
            viewport: viewport,
        }
    }

    pub fn start_loop<F>(&mut self, mut callback: F) where F: FnMut(&mut RenderContext) -> Action {
        loop {
            match callback(self) {
                Action::Stop => break,
                Action::Continue => ()
            };

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
    }

    pub fn update(&mut self, context: &GameContext) {
        self.tilemap.update(context, &self.viewport);
        self.spritemap.update(context, &self.viewport);
        self.shadowmap.update(context, &self.viewport);
        self.ui.update(context, &self.viewport);
    }

    pub fn render(&mut self) {
        let mut target = self.backend.draw();
        target.clear_color_and_depth((0.0, 0.0, 0.0, 0.0), 1.0);

        let millis = self.accumulator.millis_since_start();

        self.background.render(&self.backend, &mut target, &self.viewport);

        self.tilemap.redraw(&self.backend, millis);
        self.tilemap.render(&self.backend, &mut target, &self.viewport);

        self.spritemap.redraw(&self.backend, millis);
        self.spritemap.render(&self.backend, &mut target, &self.viewport);

        self.shadowmap.redraw(&self.backend, millis);
        self.shadowmap.render(&self.backend, &mut target, &self.viewport);

        self.ui.render(&self.backend, &mut target, &self.viewport);

        target.finish().unwrap();
    }

    pub fn set_viewport(&mut self, w: u32, h: u32) {
        self.viewport = Viewport {
            position: (0, 0),
            size: (w, h),
            scale: self.viewport.scale,
            camera: self.viewport.camera,
        };
    }

    pub fn poll_events(&self) -> Vec<glutin::Event> {
        self.backend.poll_events().collect()
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

    pub fn query<R, T: 'static + UiQuery<QueryResult=R>>(&mut self, layer: &mut T) -> Option<R> {
        loop {
            for event in self.backend.poll_events() {
                match layer.on_event(event) {
                    EventResult::Done => {
                        self.ui.render_all();
                        return layer.result();
                    },
                    EventResult::Canceled => {
                        self.ui.render_all();
                        return None;
                    },
                    _ => {
                        self.ui.render_all();
                        self.ui.draw_layer(layer);
                    }
                }
            }

            self.render();
            self.accumulator.step_frame();
        }

        None
    }
}

pub trait Renderable {
    fn render<F, S>(&self, display: &F, target: &mut S, viewport: &Viewport)
        where F: glium::backend::Facade, S: glium::Surface;
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

        let millis = util::get_duration_millis(&Instant::now().duration_since(self.start));

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
        util::get_duration_millis(&duration)
    }
}
