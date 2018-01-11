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
            window: UiWindow::new(),
            map: UiPixmap::new(tiles, size),
        }
    }
}

impl UiElement for MapLayer {
    fn draw<'a>(&self, renderer: &UiSubRenderer<'a>) {
        let pos = (120, 120);
        let size = (420, 420);
        self.window.draw(&mut renderer.sub_renderer(pos, size));
        self.map.draw(&mut renderer.sub_renderer(pos, size));
    }
}

impl UiLayer for MapLayer {
    fn on_event(&mut self, event: glutin::WindowEvent) -> EventResult {
        match event {
            glutin::WindowEvent::KeyboardInput { input, .. } => {
                if ElementState::Pressed == input.state {
                    if let Some(code) = input.virtual_keycode {
                        match code {
                            VirtualKeyCode::Escape | VirtualKeyCode::Return => {
                                return EventResult::Done
                            },
                            _ => return EventResult::Ignored,
                        }
                    }
                }
                EventResult::Ignored
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
