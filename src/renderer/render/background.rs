use glium;
use glium::backend::Facade;
use glium::index::PrimitiveType;

use renderer::render::{self, Renderable, Viewport, Vertex, QUAD_INDICES};

pub struct Background {
    vertices: glium::VertexBuffer<Vertex>,
    indices: glium::IndexBuffer<u16>,
    program: glium::Program,
}

pub const BG_QUAD: [Vertex; 4] = [
    Vertex { position: [-1,  1], },
    Vertex { position: [ 1,  1], },
    Vertex { position: [-1, -1], },
    Vertex { position: [ 1, -1], },
];

impl Background {
    pub fn new<F: Facade>(display: &F) -> Self {
        let vertices = glium::VertexBuffer::immutable(display, &BG_QUAD).unwrap();
        let indices = glium::IndexBuffer::immutable(display, PrimitiveType::TrianglesList, &QUAD_INDICES).unwrap();

        Background {
            vertices: vertices,
            indices: indices,
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
    fn render<F, S>(&self, _display: &F, target: &mut S, viewport: &Viewport)
        where F: Facade, S: glium::Surface {

        let (w, h) = (viewport.size.0 as f32, viewport.size.1 as f32);
        let scale = viewport.scale;

        let uniforms = uniform! {
            u_resolution: [w * scale, h * scale],
            u_time: 0.0,
        };

        let params = glium::DrawParameters {
            .. Default::default()
        };

        target.draw(&self.vertices,
                    &self.indices,
                    &self.program,
                    &uniforms,
                    &params).unwrap();
    }
}

