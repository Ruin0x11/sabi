use glium::glutin::{VirtualKeyCode, ElementState};

use renderer::ui::*;
use renderer::ui::elements::*;

pub struct ChoiceLayer {
    window: UiWindow,
    list: UiList,
}

impl ChoiceLayer {
    pub fn new(choices: Vec<String>) -> Self {
        ChoiceLayer {
            window: UiWindow::new((120, 120)),
            list: UiList::new((120, 120), choices),
        }
    }
}

impl UiElement for ChoiceLayer {
    fn draw<'a>(&self, renderer: &UiSubRenderer<'a>) {
        let pos = (120, 120);
        let size = (420, 420);
        self.window.draw(&mut renderer.sub_renderer(pos, size));
        self.list.draw(&mut renderer.sub_renderer(pos, size));
    }
}

impl UiLayer for ChoiceLayer {
    fn on_event(&mut self, event: glutin::Event) -> EventResult {
        match event {
            glutin::Event::KeyboardInput(ElementState::Pressed, _, Some(code)) => {
                match code {
                    VirtualKeyCode::Escape => EventResult::Canceled,
                    VirtualKeyCode::Return => EventResult::Done,
                    VirtualKeyCode::Up => {
                        self.list.select_prev();
                        EventResult::Consumed(None)
                    },
                    VirtualKeyCode::Down => {
                        self.list.select_next();
                        EventResult::Consumed(None)
                    },
                    _ => EventResult::Ignored,
                }
            },
            _ => EventResult::Ignored,
        }
    }
}

impl UiQuery for ChoiceLayer {
    type QueryResult = usize;

    fn result(&self) -> Option<usize> {
        self.list.get_selected_idx()
    }
}
