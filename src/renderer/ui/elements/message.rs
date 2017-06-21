use renderer::ui::elements::UiElement;
use renderer::ui::renderer::{TexDir, UiRenderer};
use renderer::render::{SCREEN_WIDTH, SCREEN_HEIGHT};
use renderer::render::Viewport;

const LINE_HEIGHT: usize =  16;

pub struct UiMessageLog {
    pos: (u32, u32),
    size: (u32, u32),

    messages: Vec<String>,
}

impl UiMessageLog {
    pub fn new(viewport: &Viewport) -> Self {
        UiMessageLog {
            pos: (0, viewport.height() - 120),
            size: (viewport.width(), 120),
            messages: Vec::new(),
        }
    }

    pub fn max_lines(&self) -> usize {
        self.size.1 as usize / LINE_HEIGHT
    }

    pub fn update(&mut self, messages: Vec<String>) {
        self.messages = messages;
    }
}

impl UiElement for UiMessageLog {
    fn draw(&self, renderer: &mut UiRenderer) {
        let (x, y) = self.pos;
        let (w, h) = self.size;

        renderer.with_color((128, 128, 128, 255), |r| {
            r.repeat_tex("textwin", TexDir::Area,
                         (x,     y,
                          x + w,  y + h),
                         (0, 0), (46, 45));
        });

        let (tx, ty) = (x as i32 + 8, (y + h - 8) as i32);

        let max_lines = self.max_lines();
        let mut idx = 0;

        for line in self.messages.iter() {
            if !line.is_empty() {
                for wrapped in renderer.font().wrap_text(line, w - 16) {
                    let offset = (idx * LINE_HEIGHT) as i32;

                    renderer.add_string_shadow((tx, ty - offset), None, &wrapped);

                    idx += 1;

                    if idx >= max_lines {
                        return;
                    }
                }
            }
        }
    }
}
