use std::fmt::Display;

use glium::glutin::{VirtualKeyCode, ElementState};

use renderer::ui::*;
use renderer::ui::elements::*;

pub struct InputLayer {
    win: UiWindow,
    prompt: UiText,
    text: UiText,
}

impl InputLayer {
    pub fn new(prompt: &str) -> Self {
        InputLayer {
            win: UiWindow::new((100, 100)),
            prompt: UiText::new((120, 120), prompt),
            text: UiText::new((120, 140), &""),
        }
    }
}

impl UiElement for InputLayer {
    fn draw(&self, renderer: &mut UiRenderer) {
        self.win.draw(renderer);
        self.prompt.draw(renderer);
        self.text.draw(renderer);
    }
}

impl UiLayer for InputLayer {
    fn on_event(&mut self, event: glutin::Event) -> EventResult {
        match event {
            glutin::Event::KeyboardInput(ElementState::Pressed, _, Some(code)) => {
                match code {
                    VirtualKeyCode::Escape => EventResult::Canceled,
                    VirtualKeyCode::Return => EventResult::Done,
                    VirtualKeyCode::Back => {
                        let mut t = self.text.text();
                        if !t.is_empty() {
                            t.pop();
                        }
                        self.text.set(&t);
                        EventResult::Consumed(None)
                    },
                    keycode => match keycode_to_char(keycode) {
                        Some(ch) => {
                            let mut t = self.text.text();
                            t.push(ch);
                            self.text.set(&t);
                            EventResult::Consumed(None)
                        },
                        None => EventResult::Ignored,
                    }
                }
            },
            _ => EventResult::Ignored,
        }
    }
}

impl UiQuery for InputLayer {
    type QueryResult = String;

    fn result(&self) -> Option<String> {
        Some(self.text.text())
    }
}

fn keycode_to_char(keycode: VirtualKeyCode) -> Option<char> {
    match keycode {
        VirtualKeyCode::A    => Some('a'),
        VirtualKeyCode::B    => Some('b'),
        VirtualKeyCode::C    => Some('c'),
        VirtualKeyCode::D    => Some('d'),
        VirtualKeyCode::E    => Some('e'),
        VirtualKeyCode::F    => Some('f'),
        VirtualKeyCode::G    => Some('g'),
        VirtualKeyCode::H    => Some('h'),
        VirtualKeyCode::I    => Some('i'),
        VirtualKeyCode::J    => Some('j'),
        VirtualKeyCode::K    => Some('k'),
        VirtualKeyCode::L    => Some('l'),
        VirtualKeyCode::M    => Some('m'),
        VirtualKeyCode::N    => Some('n'),
        VirtualKeyCode::O    => Some('o'),
        VirtualKeyCode::P    => Some('p'),
        VirtualKeyCode::Q    => Some('q'),
        VirtualKeyCode::R    => Some('r'),
        VirtualKeyCode::S    => Some('s'),
        VirtualKeyCode::T    => Some('t'),
        VirtualKeyCode::U    => Some('u'),
        VirtualKeyCode::V    => Some('v'),
        VirtualKeyCode::W    => Some('w'),
        VirtualKeyCode::X    => Some('x'),
        VirtualKeyCode::Y    => Some('y'),
        VirtualKeyCode::Z    => Some('z'),
        VirtualKeyCode::Key0 => Some('0'),
        VirtualKeyCode::Key1 => Some('1'),
        VirtualKeyCode::Key2 => Some('2'),
        VirtualKeyCode::Key3 => Some('3'),
        VirtualKeyCode::Key4 => Some('4'),
        VirtualKeyCode::Key5 => Some('5'),
        VirtualKeyCode::Key6 => Some('6'),
        VirtualKeyCode::Key7 => Some('7'),
        VirtualKeyCode::Key8 => Some('8'),
        VirtualKeyCode::Key9 => Some('9'),
        _ => None,
    }
}
