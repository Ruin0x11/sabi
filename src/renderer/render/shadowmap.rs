use std::collections::HashSet;

use glium;
use glium::backend::Facade;

use graphics::Mark;
use point::RectangleIter;
use renderer::render::{self, Renderable, Vertex, Viewport};

#[derive(Clone, Copy)]
struct Instance {
    map_coord: [i32; 2],
    tile_index: i8,
    color: [u8; 4],
}

implement_vertex!(Instance, map_coord, tile_index, color);

pub struct ShadowMap {
    shadows: Vec<Shadow>,
    instances: glium::VertexBuffer<Instance>,

    indices: glium::IndexBuffer<u16>,
    vertices: glium::VertexBuffer<Vertex>,
    program: glium::Program,

    valid: bool,
}

struct Shadow {
    pos: (i32, i32),
    color: (u8, u8, u8, u8),
}

impl ShadowMap {
    pub fn new<F: Facade>(display: &F) -> Self {
        let (vertices, indices) = render::make_quad_buffers(display);

        let instances = glium::VertexBuffer::immutable(display, &[]).unwrap();
        let program = render::load_program(display, "shadow.vert", "shadow.frag").unwrap();

        ShadowMap {
            shadows: Vec::new(),
            instances: instances,
            vertices: vertices,
            indices: indices,
            program: program,
            valid: false,
        }
    }

    fn make_instances<F: Facade>(&mut self, display: &F) {
        let mut instances = Vec::new();
        for shadow in self.shadows.iter() {
            let (x, y) = shadow.pos;
            let (r, g, b, a) = shadow.color;

            instances.push(Instance {
                map_coord: [x, y],
                color: [r, g, b, a],
                tile_index: 4,
            })
        }
        self.instances = glium::VertexBuffer::immutable(display, &instances).unwrap();
    }
}

impl Renderable for ShadowMap {
    fn render<F, S>(&self, _display: &F, target: &mut S, viewport: &Viewport)
    where
        F: glium::backend::Facade,
        S: glium::Surface,
    {

        let (proj, scissor) = viewport.main_window();

        let uniforms =
            uniform! {
            matrix: proj,
            tile_size: [48u32; 2],
        };

        let params = glium::DrawParameters {
            blend: glium::Blend::alpha_blending(),
            scissor: Some(scissor),
            ..Default::default()
        };

        target.draw(
            (&self.vertices, self.instances.per_instance().unwrap()),
            &self.indices,
            &self.program,
            &uniforms,
            &params,
        )
              .unwrap();
    }
}

use world::World;
use world::traits::Query;
use GameContext;
use point::Point;
use renderer::interop::RenderUpdate;

fn make_shadows(world: &World, viewport: &Viewport) -> Vec<Shadow> {
    let camera = world.flags().camera;
    let start_corner = viewport.min_tile_pos(camera);
    let area = RectangleIter::new(start_corner, Viewport::renderable_area().into());

    let mut visible = HashSet::new();
    let points: Vec<Point> = match world.player() {
        Some(player) => {
            if let Some(fov) = world.ecs().fovs.get(player) {
                fov.visible.iter().cloned().collect()
            } else {
                area.clone().collect()
            }
        },
        None => area.clone().collect(),
    };

    for point in points.iter() {
        visible.insert(*point);
    }

    let mut shadows = Vec::new();

    for point in area {
        if !visible.contains(&point) {
            let shadow = Shadow {
                pos: (point - start_corner).into(),
                color: (0, 0, 0, 128),
            };
            shadows.push(shadow);
        }
    }

    shadows
}

fn make_marks(world: &World, viewport: &Viewport) -> Vec<Shadow> {
    let camera = world.flags().camera;
    let start_corner = viewport.min_tile_pos(camera);
    let mut marks = Vec::new();

    {
        let mut add = |mark: &Mark| {
            let pos: Point = mark.pos - start_corner;
            if pos >= (0, 0) {
                let shadow = Shadow {
                    pos: pos.into(),
                    color: (mark.color.r, mark.color.g, mark.color.b, 64),
                };
                marks.push(shadow);
            }
        };

        for mark in world.marks.iter() {
            add(mark);
        }

        for mark in world.debug_overlay.iter() {
            add(mark);
        }
    }

    marks
}

fn make_map(world: &World, viewport: &Viewport) -> Vec<Shadow> {
    let mut map = Vec::new();
    let shadows = make_shadows(world, viewport);
    let marks = make_marks(world, viewport);

    map.extend(shadows);
    map.extend(marks);

    map
}

impl RenderUpdate for ShadowMap {
    fn should_update(&self, _context: &GameContext) -> bool {
        true
    }

    fn update(&mut self, context: &GameContext, viewport: &Viewport) {
        let world = &context.state.world;
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
