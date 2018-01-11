use graphics::color::Color;
use renderer::ui::traits::UiRenderable;
use renderer::ui::elements::*;

pub struct UiPixmap {
    pixels: Vec<Color>,
    size: (u32, u32),
}

impl UiPixmap {
    pub fn new(pixels: Vec<Color>, size: (u32, u32)) -> Self {
        let pixel_count = size.0 * size.1;
        assert_eq!(pixels.len(), pixel_count as usize);
        UiPixmap {
            pixels: pixels,
            size: size,
        }
    }

    pub fn get(&self, x: u32, y: u32) -> Color {
        assert!(x < self.size.0);
        assert!(y < self.size.1);

        let idx = (y * self.size.0 + x) as usize;

        self.pixels[idx]
    }
}

impl UiElement for UiPixmap {
    fn draw<'a>(&self, renderer: &UiSubRenderer<'a>) {
        for x in 0..self.size.0 {
            for y in 0..self.size.1 {
                let color = self.get(x, y);
                let px = x as i32 * 4;
                let py = y as i32 * 4;

                renderer.with_color((color.r, color.g, color.b, 255), |r| {
                    r.add_tex_stretch("pixel", (px, py, px + 8, py + 8), None, (0, 0), (1, 1));
                })
            }
        }
    }
}
