use super::renderer::*;

pub trait UiRenderable {
    fn get_font_size(&self) -> u32;

    fn text_width_px(&self, text: &str) -> u32;

    fn wrap_text(&self, text: &str, width: u32) -> Vec<String>;

    fn get_color(&self) -> (u8, u8, u8, u8);

    fn repeat_tex(&self,
                  key: &'static str,
                  dir: TexDir,
                  clipping_rect: (u32, u32, u32, u32),
                  tex_pos: (u32, u32),
                  tex_area: (u32, u32));

    fn add_tex(&self,
               key: &'static str,
               screen_pos: (i32, i32),
               clip_rect: Option<(u32, u32, u32, u32)>,
               tex_pos: (u32, u32),
               tex_area: (u32, u32));

    fn add_tex_stretch(&self,
                       key: &'static str,
                       screen_pos: (i32, i32, i32, i32),
                       clip_rect: Option<(u32, u32, u32, u32)>,
                       tex_pos: (u32, u32),
                       tex_area: (u32, u32));

    fn add_string_shadow(&self,
                         screen_pos: (i32, i32),
                         clipping_rect: Option<(u32, u32, u32, u32)>,
                         text: &str);

    fn add_string(&self,
                  screen_pos: (i32, i32),
                  clipping_rect: Option<(u32, u32, u32, u32)>,
                  text: &str);
}
