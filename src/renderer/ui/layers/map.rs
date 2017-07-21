use glium::glutin::{VirtualKeyCode, ElementState};

use renderer::ui::*;
use renderer::ui::elements::*;

use graphics::color::Color;

pub struct MapLayer {
    window: UiWindow,
    map: UiPixmap,
}

impl MapLayer {
    pub fn new(tiles: Vec<Color>, size: (u32, u32)) -> Self {
        MapLayer {
            window: UiWindow::new((120, 120)),
            map: UiPixmap::new(tiles, size),
        }
    }
}

impl UiElement for MapLayer {
    fn draw<'a>(&self, renderer: &UiSubRenderer<'a>) {
        let pos = (120, 120);
        let size = (420, 420);
        self.window.draw(&mut renderer.sub_renderer(pos, size));
        self.map.draw(&mut renderer.sub_renderer((240, 240), size));
    }
}

impl UiLayer for MapLayer {
    fn on_event(&mut self, event: glutin::Event) -> EventResult {
        match event {
            glutin::Event::KeyboardInput(ElementState::Pressed, _, Some(code)) => {
                match code {
                    VirtualKeyCode::Escape | VirtualKeyCode::Return => EventResult::Done,
                    _ => EventResult::Ignored,
                }
            },
            _ => EventResult::Ignored,
        }
    }
}

impl UiQuery for MapLayer {
    type QueryResult = ();

    fn result(&self) -> Option<()> {
        None
    }
}
