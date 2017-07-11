use std;
use renderer::ui::elements::UiElement;
use renderer::ui::renderer::UiSubRenderer;
use renderer::ui::traits::*;
use util::clamp;

pub struct UiBar {
    pos: (i32, i32),
    max: i32,
    current: i32,
    color: (u8, u8, u8, u8),
}

impl UiBar {
    pub fn new(pos: (i32, i32), max: i32, color: (u8, u8, u8, u8)) -> Self {
        UiBar {
            pos: pos,
            max: max,
            current: max / 2,
            color: color,
        }
    }

    pub fn set(&mut self, amount: i32) {
        self.current = clamp(amount, 0, self.max);
    }

    pub fn set_max(&mut self, amount: i32) {
        self.max = clamp(amount, 0, std::i32::MAX);
        self.current = clamp(amount, 0, self.max);
    }

    pub fn percent(&self) -> f32 {
        self.current as f32 / self.max as f32
    }
}


impl UiElement for UiBar {
    fn draw<'a>(&self, renderer: &mut UiSubRenderer<'a>) {
        let bar_portion = (258.0 * self.percent()) as u32;
        renderer.add_tex("bar", self.pos, None, (0, 30), (258, 30));

        renderer.with_color(self.color, |r| {
            r.add_tex("bar", self.pos, None, (0, 0), (bar_portion, 30));
        });

        let text = format!("{} / {}", self.current, self.max);
        let text_width = renderer.text_width_px(&text) as i32;

        let text_pos = (self.pos.0 + (258 / 2) - (text_width / 2),
                        self.pos.1 + 30 - (renderer.get_font_size() as i32));

        renderer.add_string_shadow(text_pos, None, &text);
    }
}
