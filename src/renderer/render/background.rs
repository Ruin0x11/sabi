use glium;
use glium::backend::Facade;

use renderer::render::{self, Renderable, Viewport};

pub struct Background {
    program: glium::Program,
}

impl Background {
    pub fn new<F: Facade>(display: &F) -> Self {
        Background {
            program: render::load_program(display, "bg.vert", "bg.frag").unwrap(),
        }
    }

    pub fn refresh_shaders<F: Facade>(&mut self, display: &F) {
        match render::load_program(display, "bg.vert", "bg.frag") {
            Ok(program) => self.program = program,
            Err(e)      => println!("Shader error: {:?}", e),
        }
    }
}

impl Renderable for Background {
    fn render<F, S>(&self, display: &F, target: &mut S, viewport: &Viewport)
        where F: Facade, S: glium::Surface {

        let (w, h) = (viewport.size.0 as f32, viewport.size.1 as f32);
        let scale = viewport.scale;

        let (vertices, indices) = render::make_quad_buffers(display);

        let uniforms = uniform! {
            u_resolution: [w * scale, h * scale],
            u_time: 0.0,
        };

        let params = glium::DrawParameters {
            .. Default::default()
        };

        target.draw(&vertices,
                    &indices,
                    &self.program,
                    &uniforms,
                    &params).unwrap();
    }
}

