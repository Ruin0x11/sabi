use renderer::ui::*;
use renderer::ui::renderer::*;

pub struct UiText {
    text_lines: Vec<String>,
}

fn split(text: &str) -> Vec<String> {
    text.split('\n').map(|s| s.to_string()).collect()
}

impl UiText {
    pub fn new(text: &str) -> Self {
        UiText { text_lines: split(text) }
    }

    pub fn set(&mut self, text: &str) {
        self.text_lines = split(text);
    }

    pub fn text(&self) -> String {
        self.text_lines.join("\n")
    }
}

impl UiElement for UiText {
    fn draw<'a>(&self, renderer: &UiSubRenderer<'a>) {
        for (idx, line) in self.text_lines.iter().enumerate() {
            let pos = (0, (idx as u32 * renderer.get_font_size()) as i32);
            renderer.with_color((0, 0, 0, 255), |r| { r.add_string(pos, None, line); });
        }
    }
}
