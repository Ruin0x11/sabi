use cgmath;
use glium;
use renderer::render::{SCREEN_WIDTH, SCREEN_HEIGHT};

#[derive(Debug)]
pub struct Viewport {
    pub position: (u32, u32),
    pub size: (u32, u32),
    pub scale: f32,
    pub camera: (i32, i32),
}

pub type RendererSubarea = ([[f32; 4]; 4], glium::Rect);

impl Viewport {
    pub fn width(&self) -> u32 {
        self.size.0
    }

    pub fn height(&self) -> u32 {
        self.size.1
    }

    pub fn main_window(&self) -> RendererSubarea {
        let (w, h) = self.scaled_size();
        self.make_subarea((0, 0, w, h - 120))
    }

    pub fn scaled_size(&self) -> (u32, u32) {
        ((self.size.0 as f32 * self.scale) as u32, (self.size.1 as f32 * self.scale) as u32)
    }

    pub fn visible_area(&self) -> (u32, u32) {
        (self.size.0 / 48, (self.size.1 - 120) / 48)
    }

    pub fn renderable_area(&self) -> (i32, i32) {
        (self.width() as i32 / 48, self.height() as i32 / 48)
    }


    /// Returns the tile position of the upper-left corner of the viewport with
    /// the given camera coordinates.
    pub fn min_tile_pos<I: Into<(i32, i32)>>(&self, camera: I) -> (i32, i32) {
        let camera = camera.into();
        let (vw, vh) = self.visible_area();
        (camera.0 - (vw as i32 / 2), camera.1 - (vh as i32 / 2))
    }

    /// Returns the tile position of the bottom-right corner of the viewport
    /// with the given camera coordinates.
    pub fn max_tile_pos<I: Into<(i32, i32)>>(&self, camera: I) -> (i32, i32) {
        let c = self.min_tile_pos(camera);
        let v = self.renderable_area();

        (c.0 + v.0, c.1 + v.1)
    }

    fn make_subarea(&self, area: (u32, u32, u32, u32)) -> RendererSubarea {
        (self.camera_projection(), self.scissor(area))
    }

    pub fn static_projection(&self) -> [[f32; 4]; 4] {
        self.make_projection_matrix((0, 0))
    }

    pub fn camera_projection(&self) -> [[f32; 4]; 4] {
        self.make_projection_matrix(self.camera)
    }

    fn make_projection_matrix(&self, offset: (i32, i32)) -> [[f32; 4]; 4] {
        let (w, h) = (self.size.0 as f32, self.size.1 as f32);
        let (x, y) = (offset.0 as f32, offset.1 as f32);

        let left = x;
        let right = x + w;
        let bottom = y + h;
        let top = y;

        cgmath::ortho(left, right, bottom, top, -1.0, 1.0).into()
    }

    fn scissor(&self, area: (u32, u32, u32, u32)) -> glium::Rect {
        let (ax, ay, aw, ah) = area;
        let (_, h) = self.scaled_size();
        let conv = |i| (i as f32 * self.scale) as u32;

        glium::Rect {
            left: conv(ax),
            bottom: conv(ay) + conv(h - ah),
            width: conv(aw - ax),
            height: conv(ah) - conv(ay * 2),
        }
    }
}
