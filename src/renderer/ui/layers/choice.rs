use std::fmt::Display;

use glium::glutin::{VirtualKeyCode, ElementState};

use GameContext;
use data::Walkability;
use engine::keys::{Key, KeyCode};
use graphics::cell::{CellFeature, StairDest, StairDir};
use logic::Action;
use logic::entity::EntityQuery;
use point::{Direction, Point};
use renderer::ui::*;
use renderer::ui::elements::*;
use world::traits::*;
use world::{self, World};

pub struct ChoiceLayer {
    list: UiList,
}

impl ChoiceLayer {
    pub fn new(choices: Vec<String>) -> Self {
        ChoiceLayer {
            list: UiList::new((120, 120), choices),
        }
    }
}

impl UiElement for ChoiceLayer {
    fn draw(&self, renderer: &mut UiRenderer) {
        self.list.draw(renderer);
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
