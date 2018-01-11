use renderer::ui::*;
use renderer::ui::renderer::*;

pub struct UiWindow {}

impl UiWindow {
    pub fn new() -> Self {
        UiWindow {}
    }
}

impl UiElement for UiWindow {
    fn draw<'a>(&self, renderer: &UiSubRenderer<'a>) {
        let (w, h) = renderer.size;

        // center
        renderer.add_tex_stretch("win", (0, 0, w as i32, h as i32), None, (0, 0), (64, 64));

        renderer.repeat_tex("win", TexDir::Area, (0, 0, w, h), (0, 64), (64, 64));

        // corners
        renderer.add_tex("win", (0, 0), None, (64, 0), (16, 16));
        renderer.add_tex("win", (0, (h - 16) as i32), None, (64, 48), (16, 16));
        renderer.add_tex("win", ((w - 16) as i32, 0), None, (112, 0), (16, 16));
        renderer.add_tex("win", ((w - 16) as i32, (h - 16) as i32), None, (112, 48), (16, 16));

        // borders
        renderer.repeat_tex("win", TexDir::Horizontal, (16, 0, (w - 16), 16), (80, 0), (16, 16));
        renderer.repeat_tex("win",
                            TexDir::Horizontal,
                            (16, (h - 16), (w - 16), h),
                            (80, 48),
                            (16, 16));
        renderer.repeat_tex("win", TexDir::Vertical, (0, 16, 16, (h - 16)), (64, 16), (16, 16));
        renderer.repeat_tex("win",
                            TexDir::Vertical,
                            ((w - 16), 16, w, (h - 16)),
                            (112, 16),
                            (16, 16));
    }
}
