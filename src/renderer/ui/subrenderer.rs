use super::renderer::*;
use super::traits::*;

pub struct UiSubRenderer<'a> {
    pub offset: (i32, i32),
    pub size: (u32, u32),
    backend: &'a UiRenderer,
}

impl<'a> UiSubRenderer<'a> {
    pub fn new(backend: &'a UiRenderer) -> Self {
        UiSubRenderer {
            offset: (0, 0),
            size: (0, 0),
            backend: backend,
        }
    }

    pub fn sub_renderer<I: Into<(i32, i32)>>(&'a self,
                                             offset: I,
                                             size: (u32, u32))
                                             -> UiSubRenderer<'a> {
        let offset = offset.into();
        UiSubRenderer {
            offset: (self.offset.0 + offset.0, self.offset.1 + offset.1),
            size: size,
            backend: self.backend,
        }
    }

    pub fn with_color<F>(&self, color: (u8, u8, u8, u8), callback: F)
    where
        F: FnOnce(&UiSubRenderer),
    {
        self.backend.with_color(color, || callback(self))
    }


    fn offset_screen_pos(&self, relative: (i32, i32)) -> (i32, i32) {
        (relative.0 + self.offset.0, relative.1 + self.offset.1)
    }

    fn offset_stretch_rect(&self, rect: (i32, i32, i32, i32)) -> (i32, i32, i32, i32) {
        (rect.0 + self.offset.0,
         rect.1 + self.offset.1,
         rect.2 + self.offset.0,
         rect.3 + self.offset.1)
    }

    fn offset_clipping_rect(&self, rect: (u32, u32, u32, u32)) -> (u32, u32, u32, u32) {
        (rect.0 + self.offset.0 as u32,
         rect.1 + self.offset.1 as u32,
         rect.2 + self.offset.0 as u32,
         rect.3 + self.offset.1 as u32)
    }
}

impl<'a> UiRenderable for UiSubRenderer<'a> {
    fn get_color(&self) -> (u8, u8, u8, u8) {
        self.backend.get_color()
    }

    fn get_font_size(&self) -> u32 {
        self.backend.get_font_size()
    }

    fn text_width_px(&self, text: &str) -> u32 {
        self.backend.text_width_px(text)
    }

    fn wrap_text(&self, text: &str, width: u32) -> Vec<String> {
        self.backend.wrap_text(text, width)
    }

    fn repeat_tex(&self,
                  key: &'static str,
                  dir: TexDir,
                  clipping_rect: (u32, u32, u32, u32),
                  tex_pos: (u32, u32),
                  tex_area: (u32, u32)) {
        let offset_clipping_rect = self.offset_clipping_rect(clipping_rect);
        self.backend
            .repeat_tex(key, dir, offset_clipping_rect, tex_pos, tex_area);
    }

    fn add_tex(&self,
               key: &'static str,
               screen_pos: (i32, i32),
               clipping_rect: Option<(u32, u32, u32, u32)>,
               tex_pos: (u32, u32),
               tex_area: (u32, u32)) {

        let offset_screen_pos = self.offset_screen_pos(screen_pos);
        let offset_clipping_rect = clipping_rect.map(|rect| self.offset_clipping_rect(rect));
        self.backend
            .add_tex(key, offset_screen_pos, offset_clipping_rect, tex_pos, tex_area);
    }

    fn add_tex_stretch(&self,
                       key: &'static str,
                       screen_rect: (i32, i32, i32, i32),
                       clipping_rect: Option<(u32, u32, u32, u32)>,
                       tex_pos: (u32, u32),
                       tex_area: (u32, u32)) {
        let offset_screen_rect = self.offset_stretch_rect(screen_rect);
        let offset_clipping_rect = clipping_rect.map(|rect| self.offset_clipping_rect(rect));
        self.backend
            .add_tex_stretch(key, offset_screen_rect, offset_clipping_rect, tex_pos, tex_area);
    }

    fn add_string_shadow(&self,
                         screen_pos: (i32, i32),
                         clipping_rect: Option<(u32, u32, u32, u32)>,
                         text: &str) {
        let offset_screen_pos = self.offset_screen_pos(screen_pos);
        let offset_clipping_rect = clipping_rect.map(|rect| self.offset_clipping_rect(rect));
        self.backend
            .add_string_shadow(offset_screen_pos, offset_clipping_rect, text);
    }

    fn add_string(&self,
                  screen_pos: (i32, i32),
                  clipping_rect: Option<(u32, u32, u32, u32)>,
                  text: &str) {
        let offset_screen_pos = self.offset_screen_pos(screen_pos);
        let offset_clipping_rect = clipping_rect.map(|rect| self.offset_clipping_rect(rect));
        self.backend
            .add_string(offset_screen_pos, offset_clipping_rect, text);
    }
}
