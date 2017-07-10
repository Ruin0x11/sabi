use std::collections::HashSet;

use glium;
use glium::backend::Facade;
use image;

use graphics::Mark;
use point::{Direction, RectangleIter};
use renderer::atlas::make_texture;
use renderer::render::{self, Renderable, Vertex, Viewport};
use super::util::*;

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
    texture: glium::texture::CompressedSrgbTexture2d,

    valid: bool,
}

#[derive(Eq, PartialEq)]
enum ShadowKind {
    Light,
    Dark,
    Marker
}

struct Shadow {
    pos: (i32, i32),
    color: (u8, u8, u8, u8),
    edges: u8,
    kind: ShadowKind,
}

fn light_tile_index(edges: u8) -> i8 {
    use point::Direction::*;
    let conn = |dir: Direction| (edges & (1 << dir_to_bit(dir))) > 0;

    // edges
    if !conn(W) && conn(E) && conn(N) && conn(S) {
        return 4;
    }
    if conn(W) && !conn(E) && conn(N) && conn(S) {
        return 6;
    }
    if conn(W) && conn(E) && !conn(N) && conn(S) {
        return 1;
    }
    if conn(W) && conn(E) && conn(N) && !conn(S) {
        return 9;
    }

    // corners
    if !conn(W) && conn(E) && !conn(N) && conn(S) {
        return 0;
    }
    if !conn(W) && conn(E) && conn(N) && !conn(S) {
        return 8;
    }
    if conn(W) && !conn(E) && !conn(N) && conn(S) {
        return 2;
    }
    if conn(W) && !conn(E) && conn(N) && !conn(S) {
        return 10;
    }

    return 3;
}

fn shadow_tile_index(edges: u8) -> i8 {
    use point::Direction::*;
    let conn = |dir: Direction| (edges & (1 << dir_to_bit(dir))) > 0;

    // edges
    if !conn(W) && conn(E) && conn(N) && conn(S) {
        return 13;
    }
    if conn(W) && !conn(E) && conn(N) && conn(S) {
        return 12;
    }
    if conn(W) && conn(E) && !conn(N) && conn(S) {
        return 14;
    }
    if conn(W) && conn(E) && conn(N) && !conn(S) {
        return 15;
    }

    // corners
    if !conn(W) && conn(E) && !conn(N) && conn(S) {
        return 17;
    }
    if !conn(W) && conn(E) && conn(N) && !conn(S) {
        return 19;
    }
    if conn(W) && !conn(E) && !conn(N) && conn(S) {
        return 16;
    }
    if conn(W) && !conn(E) && conn(N) && !conn(S) {
        return 18;
    }

    // interior corners
    if conn(W) && conn(E) && conn(N) && conn(S) {
        if !conn(SE) {
            return 25;
        }
        if !conn(SW) {
            return 26;
        }
        if !conn(NW) {
            return 27;
        }
        if !conn(NE) {
            return 24;
        }
    }

    return 11;
}

impl ShadowMap {
    pub fn new<F: Facade>(display: &F) -> Self {
        let (vertices, indices) = render::make_quad_buffers(display);

        let instances = glium::VertexBuffer::immutable(display, &[]).unwrap();
        let program = render::load_program(display, "shadow.vert", "shadow.frag").unwrap();
        let image = image::open("data/texture/shadow.png").unwrap();
        let texture = make_texture(display, image);

        ShadowMap {
            shadows: Vec::new(),
            instances: instances,
            vertices: vertices,
            indices: indices,
            program: program,
            texture: texture,
            valid: false,
        }
    }

    pub fn reload_shaders<F: Facade>(&mut self, display: &F) {
        match render::load_program(display, "shadow.vert", "shadow.frag") {
            Ok(program) => self.program = program,
            Err(e)      => println!("Shader error: {:?}", e),
        }
    }

    fn make_instances<F: Facade>(&mut self, display: &F) {
        let mut instances = Vec::new();
        for shadow in self.shadows.iter() {
            let (x, y) = shadow.pos;
            let (r, g, b, a) = shadow.color;

            let tile_index = if shadow.kind == ShadowKind::Light {
                light_tile_index(shadow.edges)
            } else {
                shadow_tile_index(shadow.edges)
            };

            instances.push(Instance {
                map_coord: [x, y],
                color: [r, g, b, a],
                tile_index: tile_index,
            })
        }
        self.instances = glium::VertexBuffer::immutable(display, &instances).unwrap();
    }
}

impl Renderable for ShadowMap {
    fn render<F, S>(&self, _display: &F, target: &mut S, viewport: &Viewport, _time: u64)
        where
        F: glium::backend::Facade,
        S: glium::Surface,
    {

        let (proj, scissor) = viewport.main_window();

        let uniforms =
            uniform! {
                matrix: proj,
                tile_size: [48u32; 2],
                tex_ratio: [(1.0/4.0) as f32, (1.0/8.0) as f32],
                tex: self.texture.sampled()
                    .wrap_function(glium::uniforms::SamplerWrapFunction::Clamp),
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

use world::{Bounds, World};
use world::traits::Query;
use state::GameState;
use point::Point;
use renderer::RenderUpdate;
use infinigen::ChunkedWorld;

fn make_shadows(world: &World, viewport: &Viewport, bound: Option<Point>) -> Vec<Shadow> {
    let camera = world.flags().camera;
    let start_corner = viewport.min_tile_pos(camera, bound);
    let area = RectangleIter::new(start_corner, viewport.renderable_area().into());

    let mut visible = HashSet::new();
    let visible_points: Vec<Point> = match world.player() {
        Some(player) => {
            if let Some(fov) = world.ecs().fovs.get(player) {
                fov.visible.iter().cloned().collect()
            } else {
                area.clone().collect()
            }
        },
        None => area.clone().collect(),
    };

    for point in visible_points.iter() {
        visible.insert(*point);
    }

    let mut shadows = Vec::new();

    let explored = &world.flags().explored;
    for pos in area {
        let edges = get_neighboring_edges(pos, |point| {
            !visible.contains(&point)
        });

        if !visible.contains(&pos) {
            let color = if !explored.contains(&pos) {
                (0, 0, 0, 255)
            } else {
                (0, 0, 0, 192)
            };

            let shadow = Shadow {
                pos: (pos - start_corner).into(),
                color: color,
                edges: edges,
                kind: ShadowKind::Dark,
            };
            shadows.push(shadow);
        } else if edges != 0 {
            // This light tile borders a shadow, so add special light->shadow border tiles
            let light_edges = !edges;

            let shadow = Shadow {
                pos: (pos - start_corner).into(),
                color: (0, 0, 0, 192),
                edges: light_edges,
                kind: ShadowKind::Light,
            };
            shadows.push(shadow);
        }
    }

    shadows
}

fn make_marks(world: &World, viewport: &Viewport, bound: Option<Point>) -> Vec<Shadow> {
    let camera = world.flags().camera;
    let start_corner = viewport.min_tile_pos(camera, bound);
    let mut marks = Vec::new();

    {
        let mut add = |mark: &Mark| {
            let pos: Point = mark.pos - start_corner;
            if pos >= (0, 0) {
                let shadow = Shadow {
                    pos: pos.into(),
                    color: (mark.color.r, mark.color.g, mark.color.b, 64),
                    edges: 0,
                    kind: ShadowKind::Marker,
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
    let bound = if let Bounds::Bounded(w, h) = *world.terrain().bounds() {
        Some(Point::new(w, h))
    } else {
        None
    };
    let shadows = make_shadows(world, viewport, bound);
    let marks = make_marks(world, viewport, bound);

    map.extend(shadows);
    map.extend(marks);

    map
}

impl RenderUpdate for ShadowMap {
    fn should_update(&self, _state: &GameState) -> bool {
        true
    }

    fn update(&mut self, state: &GameState, viewport: &Viewport) {
        let world = &state.world;
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
