use std::cell::RefCell;
use std::collections::HashMap;
use std::fs::File;
use std::path::Path;

use glium;
use glium::backend::Facade;
use glium::index::PrimitiveType;
use glium::Rect;

use graphics::color::Color;
use renderer::atlas::AtlasRect;
use renderer::atlas::font::FontTexture;
use renderer::atlas::texture_atlas::*;
use renderer::render::{self, Renderable, Viewport};
use super::traits::*;
pub use super::subrenderer::UiSubRenderer;

#[derive(Clone, Copy, Debug)]
pub struct AreaRect {
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
}

struct UiDrawList {
    commands: Vec<UiDrawCmd>,
    vertices: Vec<UiVertex>,
    indices: Vec<u16>,
}

impl UiDrawList {
    pub fn new() -> Self {
        UiDrawList {
            commands: Vec::new(),
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        self.commands.clear();
        self.vertices.clear();
        self.indices.clear();
    }

    pub fn add_command(&mut self, cmd: UiDrawCmd) {
        if self.commands.is_empty() {
            self.commands.push(cmd);
            return;
        }

        let should_merge = {
            let last = &self.commands[self.commands.len() - 1];
            last.is_text == cmd.is_text && last.clip_rect == cmd.clip_rect
        };

        if should_merge {
            let last_mut = self.commands.last_mut().unwrap();
            last_mut.elem_count += cmd.elem_count;
        } else {
            self.commands.push(cmd);
        }
    }
}

#[derive(Clone, Copy)]
struct UiDrawCmd {
    elem_count: usize,
    is_text: bool,
    clip_rect: Option<(f32, f32, f32, f32)>,
}

#[derive(Clone, Copy)]
struct UiVertex {
    pos: [f32; 2],
    tex_coords: [f32; 2],
    color: [u8; 4],
}

implement_vertex!(UiVertex, pos, tex_coords, color);

fn calc_tex_subarea(area: &AtlasRect,
                    tex_pos: (u32, u32),
                    tex_area: (u32, u32),
                    tex_size: (u32, u32))
                    -> AreaRect {

    let (tw, th) = tex_size;

    let (ratio_x, ratio_y) = (area.w as f32 / tw as f32, area.h as f32 / th as f32);

    let xa = (area.x + tex_pos.0) as f32;
    let ya = (area.y + tex_pos.1) as f32;
    let xb = xa + tex_area.0 as f32;
    let yb = ya + tex_area.1 as f32;
    let (w, h) = (area.w as f32, area.h as f32);

    AreaRect {
        x1: (xa / w) * ratio_x,
        y1: 1.0 - (ya / h) * ratio_y,
        x2: (xb / w) * ratio_x,
        y2: 1.0 - (yb / h) * ratio_y,
    }
}

pub enum TexDir {
    Horizontal,
    Vertical,
    Area,
}

pub enum TexKind {
    Elem(&'static str, (u32, u32), (u32, u32)),
    Font(AreaRect),
}

pub struct UiRenderer {
    ui_atlas: TextureAtlas,
    font: FontTexture,
    program: glium::Program,
    font_program: glium::Program,
    tile_colors: HashMap<usize, Color>,

    draw_list: RefCell<UiDrawList>,
    color_stack: RefCell<Vec<(u8, u8, u8, u8)>>,
}

fn build_ui_atlas<F: Facade>(display: &F) -> TextureAtlas {
    TextureAtlasBuilder::new()
        .add_texture("pixel")
        .add_texture("win")
        .add_texture("textwin")
        .add_texture("bar")
        .add_texture("mikan")
        .build(display)
}

impl UiRenderer {
    pub fn new<F: Facade>(display: &F) -> Self {
        let font_size = 14;

        let font = FontTexture::new(display,
                                    File::open(&Path::new("data/gohufont-14.ttf")).unwrap(),
                                    font_size,
                                    FontTexture::ascii_character_list())
                   .unwrap();

        let atlas = build_ui_atlas(display);
        let program = render::load_program(display, "identity.vert", "identity.frag").unwrap();
        let font_program = render::load_program(display, "identity.vert", "font.frag").unwrap();

        UiRenderer {
            ui_atlas: atlas,
            font: font,
            program: program,
            font_program: font_program,
            tile_colors: HashMap::new(),
            draw_list: RefCell::new(UiDrawList::new()),
            color_stack: RefCell::new(Vec::new()),
        }
    }

    pub fn clear(&self) {
        self.draw_list.borrow_mut().clear();
    }

    pub fn font(&self) -> &FontTexture {
        &self.font
    }

    pub fn with_color<F>(&self, color: (u8, u8, u8, u8), callback: F)
    where
        F: FnOnce(),
    {
        self.color_stack.borrow_mut().push(color);
        callback();
        self.color_stack.borrow_mut().pop();
    }

    pub fn add_tex_internal(&self,
                            kind: TexKind,
                            screen_pos: (i32, i32, i32, i32),
                            clip_rect: Option<(u32, u32, u32, u32)>,
                            color: (u8, u8, u8, u8)) {
        let tex_coords = match kind {
            TexKind::Elem(key, tex_pos, tex_area) => {
                let atlas_area = self.ui_atlas.get_texture_area(key);
                let tex_size = self.ui_atlas.get_texture().dimensions();
                calc_tex_subarea(atlas_area, tex_pos, tex_area, tex_size)
            },
            TexKind::Font(coords) => coords,
        };

        let is_text = match kind {
            TexKind::Elem(..) => false,
            TexKind::Font(..) => true,
        };

        let clip_rect = match clip_rect {
            Some(r) => Some((r.0 as f32, r.1 as f32, r.2 as f32, r.3 as f32)),
            None => None,
        };

        let cmd = UiDrawCmd {
            elem_count: 6,
            is_text: is_text,
            clip_rect: clip_rect,
        };

        let (sxa, sya, sxb, syb) = screen_pos;

        let color = [color.0, color.1, color.2, color.3];

        // 0---3
        // |\  |
        // | \ |
        // |  \|
        // 1---2

        let vertices = vec![
            UiVertex {
                pos: [sxa as f32, sya as f32],
                tex_coords: [tex_coords.x1, tex_coords.y1],
                color: color,
            },
            UiVertex {
                pos: [sxa as f32, syb as f32],
                tex_coords: [tex_coords.x1, tex_coords.y2],
                color: color,
            },
            UiVertex {
                pos: [sxb as f32, syb as f32],
                tex_coords: [tex_coords.x2, tex_coords.y2],
                color: color,
            },
            UiVertex {
                pos: [sxb as f32, sya as f32],
                tex_coords: [tex_coords.x2, tex_coords.y1],
                color: color,
            },
        ];

        let next_indices = |i| vec![i, i + 1, i + 2, i, i + 2, i + 3];

        let indices = next_indices(self.draw_list.borrow().vertices.len() as u16);

        self.draw_list.borrow_mut().vertices.extend(vertices);
        self.draw_list.borrow_mut().indices.extend(indices);

        // Between a draw call for every texture and merged draw calls, it is a
        // nearly 800% speed difference (or more).

        // self.draw_list.commands.push(cmd);
        self.draw_list.borrow_mut().add_command(cmd);
    }

    // Returns the width of the character that was printed in EMs.
    fn add_char(&self,
                screen_pos: (i32, i32),
                clipping_rect: Option<(u32, u32, u32, u32)>,
                total_text_width: f32,
                color: (u8, u8, u8, u8),
                ch: char)
                -> f32 {
        let glyph = match self.font.find_glyph(ch) {
            Some(glyph) => glyph,
            None => return 0.0,
        };

        let area = AreaRect {
            x1: glyph.tex_coords.0,
            y1: glyph.tex_coords.1,
            x2: glyph.tex_coords.0 + glyph.tex_size.0,
            y2: glyph.tex_coords.1 + glyph.tex_size.1,
        };

        let pt = self.font.get_font_size() as f32;

        let (ch_width, ch_height) = ((glyph.size.0 * pt) as u32, (glyph.size.1 * pt) as u32);

        // check overflow
        // if screen_pos.1 < (glyph.height_over_line * pt) as i32 {
        //     return added_width;
        // }

        let (sx, sy) = (screen_pos.0 + ((total_text_width + glyph.left_padding) * pt) as i32,
                        screen_pos.1 - (glyph.height_over_line * pt) as i32);

        let true_pos = (sx, sy, sx + ch_width as i32, sy + ch_height as i32);

        self.add_tex_internal(TexKind::Font(area), true_pos, clipping_rect, color);

        glyph.size.0 + glyph.left_padding + glyph.right_padding
    }
}

impl UiRenderable for UiRenderer {
    fn get_font_size(&self) -> u32 {
        self.font.get_font_size()
    }

    fn get_color(&self) -> (u8, u8, u8, u8) {
        match self.color_stack.borrow().last() {
            Some(color) => *color,
            None => (255, 255, 255, 255),
        }
    }

    fn text_width_px(&self, text: &str) -> u32 {
        self.font().text_width_px(text)
    }

    fn wrap_text(&self, text: &str, width: u32) -> Vec<String> {
        self.font().wrap_text(text, width)
    }

    fn repeat_tex(&self,
                  key: &'static str,
                  dir: TexDir,
                  clipping_rect: (u32, u32, u32, u32),
                  tex_pos: (u32, u32),
                  tex_area: (u32, u32)) {
        let (cxa, cya, cxb, cyb) = clipping_rect;
        let clipping_width = cxb - cxa;
        let clipping_height = cyb - cya;

        let (tw, th) = tex_area;
        let repeats_h;
        let repeats_v;
        match dir {
            TexDir::Horizontal => {
                repeats_h = clipping_width / tw;
                repeats_v = 0;
            },
            TexDir::Vertical => {
                repeats_h = 0;
                repeats_v = clipping_height / th;
            },
            TexDir::Area => {
                repeats_h = clipping_width / tw;
                repeats_v = clipping_height / th;
            },
        }

        let mut x = cxa as i32;
        let mut y = cya as i32;
        let tw = tw as i32;
        let th = th as i32;

        for _ in 0..(repeats_h + 1) {
            for _ in 0..(repeats_v + 1) {
                let screen_pos = (x, y, x + tw, y + th);
                let color = self.get_color();

                self.add_tex_internal(TexKind::Elem(key, tex_pos, tex_area),
                                      screen_pos,
                                      Some(clipping_rect),
                                      color);

                y += th;
            }
            x += tw;
            y = cya as i32;
        }
    }

    fn add_tex(&self,
               key: &'static str,
               screen_pos: (i32, i32),
               clip_rect: Option<(u32, u32, u32, u32)>,
               tex_pos: (u32, u32),
               tex_area: (u32, u32)) {
        let (sx, sy) = screen_pos;
        let (tw, th) = tex_area;

        let true_screen_pos = (sx, sy, sx + tw as i32, sy + th as i32);
        let color = self.get_color();

        self.add_tex_internal(TexKind::Elem(key, tex_pos, tex_area),
                              true_screen_pos,
                              clip_rect,
                              color);
    }

    fn add_tex_stretch(&self,
                       key: &'static str,
                       screen_pos: (i32, i32, i32, i32),
                       clip_rect: Option<(u32, u32, u32, u32)>,
                       tex_pos: (u32, u32),
                       tex_area: (u32, u32)) {
        let color = self.get_color();

        self.add_tex_internal(TexKind::Elem(key, tex_pos, tex_area), screen_pos, clip_rect, color);
    }

    fn add_string_shadow(&self,
                         screen_pos: (i32, i32),
                         clipping_rect: Option<(u32, u32, u32, u32)>,
                         text: &str) {
        let shadow_pos = (screen_pos.0 + 1, screen_pos.1 + 1);
        let color = self.get_color();

        self.with_color((0, 0, 0, 255), || { self.add_string(shadow_pos, clipping_rect, text); });
        self.with_color(color, || { self.add_string(screen_pos, clipping_rect, text); });
    }

    fn add_string(&self,
                  screen_pos: (i32, i32),
                  clipping_rect: Option<(u32, u32, u32, u32)>,
                  text: &str) {
        if text.is_empty() {
            return;
        }
        let color = self.get_color();

        let mut total_text_width = 0.0;
        for ch in text.chars() {
            // FIXME: apparently wrong, but only thing stable
            let added_width_ems =
                self.add_char(screen_pos, clipping_rect, total_text_width, color, ch);
            total_text_width += added_width_ems;
        }
    }
}

fn make_scissor(clip_rect: (f32, f32, f32, f32), height: f32, scale: f32) -> Rect {
    let conv = |i| (i * scale) as u32;
    Rect {
        left: conv(clip_rect.0),
        bottom: conv(height - clip_rect.3),
        width: conv(clip_rect.2 - clip_rect.0),
        height: conv(clip_rect.3 - clip_rect.1),
    }
}

impl<'a> Renderable for UiRenderer {
    fn render<F, S>(&self, display: &F, target: &mut S, viewport: &Viewport, _time: u64)
    where
        F: glium::backend::Facade,
        S: glium::Surface,
    {

        let proj = viewport.static_projection();
        // TODO: move into struct
        let vertices = glium::VertexBuffer::dynamic(display, &self.draw_list.borrow().vertices)
            .unwrap();
        let height = viewport.size.1 as f32;
        let mut idx_start = 0;

        for cmd in self.draw_list.borrow().commands.iter() {
            let idx_end = idx_start + cmd.elem_count;

            let indices = glium::IndexBuffer::dynamic(display,
                                                      PrimitiveType::TrianglesList,
                                                      &self.draw_list.borrow().indices
                                                          [idx_start..idx_end])
                          .unwrap();
            idx_start = idx_end;

            let texture = if cmd.is_text {
                self.font.get_texture()
            } else {
                self.ui_atlas.get_texture()
            };

            let uniforms =
                uniform! {
                matrix: proj,
                tex: texture.sampled()
                    .wrap_function(glium::uniforms::SamplerWrapFunction::Clamp)
                    .minify_filter(glium::uniforms::MinifySamplerFilter::Nearest)
                    .magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest),
            };

            let scissor = cmd.clip_rect
                             .map(|rect| make_scissor(rect, height, viewport.scale));

            let params = glium::DrawParameters {
                blend: glium::Blend::alpha_blending(),
                scissor: scissor,
                ..Default::default()
            };

            if cmd.is_text {
                target.draw(&vertices, &indices, &self.font_program, &uniforms, &params)
                      .unwrap();
            } else {
                target.draw(&vertices, &indices, &self.program, &uniforms, &params)
                      .unwrap();
            }
        }
    }
}
