use glium;
use glium::backend::Facade;
use glium::index::PrimitiveType;
use cgmath;

use point::Point;
use renderer::atlas::*;
use renderer::render::{self, Renderable, Viewport, Vertex, QUAD, QUAD_INDICES};

#[derive(Copy, Clone)]
struct Instance {
    tile_idx: usize,
    map_coord: [u32; 2],
    tex_offset: [f32; 2],
    tex_ratio: [f32; 2],
    sprite_size: [u32; 2],
}

implement_vertex!(Instance, map_coord, tex_offset, tex_ratio, sprite_size);

pub struct SpriteMap {
    sprites: Vec<(DrawSprite, Point)>,

    indices: glium::IndexBuffer<u16>,
    vertices: glium::VertexBuffer<Vertex>,
    instances: Vec<glium::VertexBuffer<Instance>>,
    program: glium::Program,

    tile_atlas: TileAtlas,
    valid: bool,
}

struct DrawSprite {
    kind: String,
}

impl SpriteMap {
    pub fn new<F: Facade>(display: &F) -> Self {
        let tile_atlas = TileAtlas::from_config(display, "data/sprites.toml");

        let (vertices, indices) = render::make_quad_buffers(display);

        let program = render::load_program(display, "sprite.vert", "sprite.frag").unwrap();

        let mut spritemap = SpriteMap {
            sprites: Vec::new(),
            indices: indices,
            vertices: vertices,
            instances: Vec::new(),
            program: program,
            tile_atlas: tile_atlas,
            valid: false,
        };

        spritemap.redraw(display, 0);
        spritemap
    }

    fn make_instances<F>(&mut self, display: &F, msecs: u64)
        where F: glium::backend::Facade {

        let mut instances = Vec::new();

        for pass in 0..self.tile_atlas.passes() {
            let data = self.sprites.iter()
                .filter(|&&(ref sprite, _)| {
                    let texture_idx = self.tile_atlas.get_tile_texture_idx(&sprite.kind);
                    texture_idx == pass
                })
                .map(|&(ref sprite, c)| {
                    let (x, y) = (c.x, c.y);
                    let (tx, ty) = self.tile_atlas.get_texture_offset(&sprite.kind, msecs);
                    let (sx, sy) = self.tile_atlas.get_tile_texture_size(&sprite.kind);
                    let tex_ratio = self.tile_atlas.get_sprite_tex_ratio(&sprite.kind);

                    // To store the tile kind without putting a string in the
                    // index vertex, a mapping from a numeric index to a string
                    // is used in the tile atlas. Then, when the tile kidn needs
                    // to be checked, the numeric index can retrieve a tile kind.
                    let tile_idx = self.tile_atlas.get_tile_index(&sprite.kind);

                    Instance { tile_idx: tile_idx,
                               map_coord: [x as u32, y as u32],
                               tex_offset: [tx, ty],
                               tex_ratio: tex_ratio,
                               sprite_size: [sx, sy], }
                }).collect::<Vec<Instance>>();
            instances.push(glium::VertexBuffer::dynamic(display, &data).unwrap());
        }

        self.instances = instances;
    }

    fn update_instances(&mut self, msecs:u64) {
        for buffer in self.instances.iter_mut() {
            for instance in buffer.map().iter_mut() {
                let (tx, ty) = self.tile_atlas.get_texture_offset_indexed(instance.tile_idx, msecs);

                instance.tex_offset = [tx, ty];
            }
        }
    }
}

impl<'a> Renderable for SpriteMap {
    fn render<F, S>(&self, display: &F, target: &mut S, viewport: &Viewport, msecs: u64)
        where F: glium::backend::Facade, S: glium::Surface {

        let (proj, scissor) = viewport.main_window();

        for pass in 0..self.tile_atlas.passes() {
            let texture = self.tile_atlas.get_texture(pass);

            let uniforms = uniform! {
                matrix: proj,
                tile_size: [48u32; 2],
                tex: texture.sampled()
                    .wrap_function(glium::uniforms::SamplerWrapFunction::Clamp)
                    .minify_filter(glium::uniforms::MinifySamplerFilter::Nearest)
                    .magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest),
            };

            let params = glium::DrawParameters {
                blend: glium::Blend::alpha_blending(),
                scissor: Some(scissor),
                .. Default::default()
            };

            let instances = self.instances.get(pass).unwrap();

            target.draw((&self.vertices, instances.per_instance().unwrap()),
                        &self.indices,
                        &self.program,
                        &uniforms,
                        &params).unwrap();
        }
    }
}

use world::EcsWorld;
use world::traits::{Query, WorldQuery};
use ecs::traits::ComponentQuery;
use GameContext;
use renderer::interop::RenderUpdate;
use renderer::render::{SCREEN_WIDTH, SCREEN_HEIGHT};

fn make_sprites(world: &EcsWorld) -> Vec<(DrawSprite, Point)> {
    let mut res = Vec::new();
    for entity in world.entities() {
        if let Some(pos) = world.position(*entity) {
            if let Some(appearance) = world.ecs().appearances.get(*entity) {
                let sprite = DrawSprite {
                    kind: appearance.kind.clone()
                };
                res.push((sprite, pos));
            }
        }
    }
    res
}

impl RenderUpdate for SpriteMap {
    fn should_update(&self, context: &GameContext) -> bool {
        true
    }

    fn update(&mut self, context: &GameContext, viewport: &Viewport) {
        let ref world = context.state.world;
        self.sprites = make_sprites(world);
        self.valid = false;
    }

    fn redraw<F: Facade>(&mut self, display: &F, msecs: u64) {
        if !self.valid {
            self.make_instances(display, msecs);
            self.valid = true;
        } else {
            self.update_instances(msecs);
        }
    }
}