use std::collections::HashSet;

use cgmath;
use glium;
use glium::backend::Facade;
use glium::index::PrimitiveType;

use point::Point;
use point::RectangleIter;
use renderer::render::{self, Renderable, Vertex, Viewport, QUAD, QUAD_INDICES};

#[derive(Clone, Copy)]
struct Instance {
    map_coord: [i32; 2],
    tile_index: i8,
}

implement_vertex!(Instance, map_coord, tile_index);

pub struct ShadowMap {
    shadows: HashSet<Point>,
    instances: glium::VertexBuffer<Instance>,

    indices: glium::IndexBuffer<u16>,
    vertices: glium::VertexBuffer<Vertex>,
    program: glium::Program,

    valid: bool,
}

impl ShadowMap {
    pub fn new<F: Facade>(display: &F, area: RectangleIter, visible: HashSet<Point>) -> Self {
        let (vertices, indices) = render::make_quad_buffers(display);

        let instances = glium::VertexBuffer::immutable(display, &[]).unwrap();
        let program = render::load_program(display, "shadow.vert", "shadow.frag").unwrap();

        ShadowMap {
            shadows: HashSet::new(),
            instances: instances,
            vertices: vertices,
            indices: indices,
            program: program,
            valid: false,
        }
    }

    fn make_instances<F: Facade>(&mut self, display: &F)  {
        let mut instances = Vec::new();
        for point in self.shadows.iter() {
            instances.push(Instance {
                map_coord: [point.x, point.y],
                tile_index: 4,
            })
        }
        self.instances = glium::VertexBuffer::immutable(display, &instances).unwrap();
    }

}

impl Renderable for ShadowMap {
    fn render<F, S>(&self, display: &F, target: &mut S, viewport: &Viewport, msecs: u64)
        where F: glium::backend::Facade, S: glium::Surface {

        let (proj, scissor) = viewport.main_window();

        let uniforms = uniform! {
            matrix: proj,
            tile_size: [48u32; 2],
        };

        let params = glium::DrawParameters {
            blend: glium::Blend::alpha_blending(),
            scissor: Some(scissor),
            .. Default::default()
        };

        target.draw((&self.vertices, self.instances.per_instance().unwrap()),
                    &self.indices,
                    &self.program,
                    &uniforms,
                    &params).unwrap();
    }
}

use world::EcsWorld;
use world::traits::Query;
use GameContext;
use point::CircleIter;
use renderer::interop::RenderUpdate;

fn make_map(world: &EcsWorld, viewport: &Viewport) -> HashSet<Point> {
    let camera = world.flags().camera;
    let start_corner = viewport.camera_tile_pos(camera);
    println!("start: {:?}", start_corner);
    let area = RectangleIter::new(start_corner, Viewport::renderable_area().into());

    let mut visible = HashSet::new();
    for point in CircleIter::new(camera, 5) {
        visible.insert(point);
    }

    let mut shadows = HashSet::new();

    for point in area {
        if !visible.contains(&point) {
            shadows.insert(point);
        }
    }

    shadows
}

impl RenderUpdate for ShadowMap {
    fn should_update(&self, context: &GameContext) -> bool {
        true
    }

    fn update(&mut self, context: &GameContext, viewport: &Viewport) {
        let ref world = context.state.world;
        self.shadows = make_map(world, viewport);
        self.valid = false;
    }

    fn redraw<F: Facade>(&mut self, display: &F, _msecs: u64) {
        if !self.valid {
            self.make_instances(display);
            self.valid = true;
        }
    }
}
