use glium::glutin::ElementState;

use renderer::ui::*;
use renderer::ui::elements::*;

use point::Point;
use logic::quest::Quest;
use graphics::color::Color;

pub struct QuestLayer {
    window: UiWindow,
    map: UiPixmap,
    list: UiList,
    description: UiText,

    desc_list: Vec<String>,
    pos_list: Vec<Point>,
}

impl QuestLayer {
    pub fn new(quests: Vec<Quest>, tiles: Vec<Color>, size: (u32, u32)) -> Self {
        let descs: Vec<String> = quests.iter().map(|q| format!("{}", q)).collect();
        let positions = quests.iter().map(|q| q.location.overworld_pos()).collect();
        let mut choices = Vec::new();
        for i in 0..descs.len() {
            choices.push(format!("Quest {}", i));
        }

        let mut l = QuestLayer {
            window: UiWindow::new(),
            map: UiPixmap::new(tiles, size),
            list: UiList::new(choices),
            description: UiText::new(String::new()),
            desc_list: descs,
            pos_list: positions,
        };

        l.refresh_text();
        l
    }

    pub fn refresh_text(&mut self) {
        self.description.text = self.desc_list[self.list.get_selected_idx().unwrap()].clone();
    }
}

impl UiElement for QuestLayer {
    fn draw<'a>(&self, renderer: &UiSubRenderer<'a>) {
        let pos = (120, 120);
        let size = (420, 420);
        self.window.draw(&mut renderer.sub_renderer(pos, size));
        self.map
            .draw(&mut renderer.sub_renderer((240, 140), (128, 128)));
        self.list.draw(&mut renderer.sub_renderer(pos, size));
        self.description
            .draw(&mut renderer.sub_renderer((240, 420), (240, 240)));
    }
}

impl UiLayer for QuestLayer {
    fn on_event(&mut self, event: glutin::WindowEvent) -> EventResult {
        match event {
            glutin::WindowEvent::KeyboardInput { input, .. } => {
                if ElementState::Pressed == input.state {
                    if let Some(code) = input.virtual_keycode {
                        let res = UiList::update(&code, &mut self.list);
                        println!("ref");
                        self.refresh_text();
                        return res;
                    }
                }
                EventResult::Ignored
            },
            _ => EventResult::Ignored,
        }
    }
}

impl UiQuery for QuestLayer {
    type QueryResult = usize;

    fn result(&self) -> Option<usize> {
        self.list.get_selected_idx()
    }
}
