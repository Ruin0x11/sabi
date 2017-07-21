use renderer::ui::*;
use renderer::ui::renderer::*;
use renderer::ui::elements::UiText;

pub struct UiList {
    items: Vec<UiText>,
    selected: usize,
}

impl UiList {
    pub fn new(pos: (u32, u32), items: Vec<String>) -> Self {
        let mut text_items = Vec::new();
        for item in items.into_iter() {
            let text = UiText::new(&item);
            text_items.push(text);
        }

        UiList {
            items: text_items,
            selected: 0,
        }
    }

    pub fn select_next(&mut self) {
        if self.items.is_empty() {
            return;
        }

        if self.selected == self.items.len() - 1 {
            return;
        }

        self.selected += 1;
    }

    pub fn select_prev(&mut self) {
        if self.items.is_empty() {
            return;
        }

        if self.selected == 0 {
            return;
        }

        self.selected -= 1;
    }

    pub fn get_selected(&self) -> Option<&UiText> {
        if self.items.is_empty() {
            return None;
        }

        self.items.get(self.selected)
    }

    pub fn get_selected_idx(&self) -> Option<usize> {
        if self.items.is_empty() {
            return None;
        }

        Some(self.selected)
    }
}

impl UiElement for UiList {
    fn draw<'a>(&self, renderer: &UiSubRenderer<'a>) {
        let item_height = 20;
        for (idx, item) in self.items.iter().enumerate() {
            let pos = (32, 32 + (item_height * idx as u32) as i32);
            println!("pos {:?}", pos);
            item.draw(&renderer.sub_renderer(pos, (0, 0)));
        }
        if let Some(idx) = self.get_selected_idx() {
            renderer.add_tex("win",
                             (16, (item_height * (idx + 1) as u32) as i32),
                             None,
                             (96, 24),
                             (16, 16));
        }
    }
}
