use glium;
use glium::glutin;
use glium::backend::Facade;

use renderer::render::{Renderable, Viewport};

pub mod elements;
mod layer;
mod renderer;

pub use self::elements::{UiElement};
pub use self::renderer::UiRenderer;
pub use self::layer::{EventResult, UiLayer, UiQuery};

use renderer::ui::elements::{UiBar, UiMessageLog};
use renderer::render::SCREEN_HEIGHT;

pub struct MainLayer {
    pub log: UiMessageLog,
    pub bar: UiBar,
}

impl MainLayer {
    pub fn new() -> Self {
        MainLayer {
            log: UiMessageLog::new(),
            bar: UiBar::new((100, SCREEN_HEIGHT as i32 - 140), 100, (255, 64, 64, 255)),
        }
    }
}

impl UiElement for MainLayer {
    fn draw(&self, renderer: &mut UiRenderer) {
        self.log.draw(renderer);
        self.bar.draw(renderer);
    }
}

impl UiLayer for MainLayer {
    fn on_event(&mut self, _event: glutin::Event) -> EventResult {
        EventResult::Ignored
    }
}

pub struct Ui {
    renderer: UiRenderer,
    valid: bool,
    layers: Vec<Box<UiLayer>>,
    pub main_layer: MainLayer,
}

impl Ui {
    pub fn new<F: Facade>(display: &F) -> Self {
        Ui {
            renderer: UiRenderer::new(display),
            valid: false,
            layers: Vec::new(),
            main_layer: MainLayer::new(),
        }
    }

    pub fn draw_layer<T: 'static + UiLayer>(&mut self, layer: &T) {
        layer.draw(&mut self.renderer)
    }

    pub fn push_layer<T: 'static + UiLayer>(&mut self, layer: T) {
        self.layers.push(Box::new(layer));
        self.invalidate();
    }

    pub fn pop_layer(&mut self) {
        self.layers.pop();
        self.invalidate();
    }

    pub fn clear(&mut self) {
        self.renderer.clear();
        self.valid = true;
    }

    pub fn on_event(&mut self, event: glutin::Event) {
        let result = match self.layers.last_mut() {
            None => EventResult::Ignored,
            Some(layer) => layer.on_event(event),
        };

        match result {
            EventResult::Ignored => (),
            EventResult::Consumed(callback) => {
                self.invalidate();
                match callback {
                    None => (),
                    Some(cb) => cb(self)
                }
            }
            EventResult::Done => self.pop_layer(),
        }
    }

    pub fn render_all(&mut self) {
            self.renderer.clear();

            println!("Draw main");
            self.main_layer.draw(&mut self.renderer);
            println!("Draw next");

            for layer in self.layers.iter() {
                layer.draw(&mut self.renderer);
            }
    }

    fn redraw(&mut self) {
        if !self.valid {
            self.render_all();
            self.valid = true;
        }
    }

    pub fn invalidate(&mut self) {
        self.valid = false;
        println!("invalidate ui");
    }
}

impl<'a> Renderable for Ui {
    fn render<F, S>(&self, display: &F, target: &mut S, viewport: &Viewport)
        where F: glium::backend::Facade, S: glium::Surface {

        self.renderer.render(display, target, viewport);
    }
}

pub struct UiWindow {
    pos: (u32, u32),
    size: (u32, u32),
}

impl UiWindow {
    pub fn new(pos: (u32, u32)) -> Self {
        UiWindow {
            pos: pos,
            size: (300, 400),
        }
    }
}

use renderer::interop::RenderUpdate;
use world::traits::*;
use GameContext;

impl RenderUpdate for Ui {
    fn should_update(&self, _context: &GameContext) -> bool {
        true
    }

    fn update(&mut self, context: &GameContext, _viewport: &Viewport) {
        let world = &context.state.world;
        if let Some(player) = world.player() {
            if let Some(health) = world.ecs().healths.get(player) {
                self.main_layer.bar.set_max(health.max_hit_points);
                self.main_layer.bar.set(health.hit_points);
            }
        }
        self.invalidate();
        self.redraw();
    }
}
