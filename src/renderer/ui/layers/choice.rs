use glium::glutin::ElementState;

use renderer::ui::*;
use renderer::ui::elements::*;

pub struct ChoiceLayer {
    window: UiWindow,
    list: UiList,
}

impl ChoiceLayer {
    pub fn new(choices: Vec<String>) -> Self {
        ChoiceLayer {
            window: UiWindow::new(),
            list: UiList::new(choices),
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
    fn on_event(&mut self, event: glutin::WindowEvent) -> EventResult {
        match event {
            glutin::WindowEvent::KeyboardInput { input, .. } => {
                if ElementState::Pressed == input.state {
                    if let Some(code) = input.virtual_keycode {
                        let res = UiList::update(&code, &mut self.list);
                        return res;
                    }
                }
                EventResult::Ignored
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
