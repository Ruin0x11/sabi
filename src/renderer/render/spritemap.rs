use glium;
use glium::backend::Facade;

use point::Point;
use renderer::atlas::*;
use renderer::render::{self, Renderable, Viewport, Vertex};

#[derive(Copy, Clone, Debug)]
struct Instance {
    tile_idx: usize,
    map_coord: [u32; 2],
    tex_offset: [f32; 2],
    tex_ratio: [f32; 2],
    sprite_size: [u32; 2],
}

implement_vertex!(Instance, map_coord, tex_offset, tex_ratio, sprite_size);

pub struct SpriteMap {
    sprites: Vec<(DrawSprite, (u32, u32))>,

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
                .map(|&(ref sprite, pos)| {
                    let (x, y) = pos;

                    let (tx, ty) = self.tile_atlas.get_texture_offset(&sprite.kind, msecs);
                    let (sx, sy) = self.tile_atlas.get_tile_texture_size(&sprite.kind);
                    let tex_ratio = self.tile_atlas.get_sprite_tex_ratio(&sprite.kind);

                    // To store the tile kind without putting a string in the
                    // index vertex, a mapping from a numeric index to a string
                    // is used in the tile atlas. Then, when the tile kidn needs
                    // to be checked, the numeric index can retrieve a tile kind.
                    let tile_idx = self.tile_atlas.get_tile_index(&sprite.kind);

                    Instance { tile_idx: tile_idx,
                               map_coord: [x, y],
                               tex_offset: [tx, ty],
                               tex_ratio: tex_ratio,
                               sprite_size: [sx, sy], }
                }).collect::<Vec<Instance>>();
            instances.push(glium::VertexBuffer::dynamic(display, &data).unwrap());
        }

        self.instances = instances;
    }

    fn update_instances(&mut self, msecs: u64) {
        for buffer in self.instances.iter_mut() {
            for instance in buffer.map().iter_mut() {
                let (tx, ty) = self.tile_atlas.get_texture_offset_indexed(instance.tile_idx, msecs);

                instance.tex_offset = [tx, ty];
            }
        }
    }
}

impl<'a> Renderable for SpriteMap {
    fn render<F, S>(&self, _display: &F, target: &mut S, viewport: &Viewport)
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

            let instances = &self.instances[pass];

            target.draw((&self.vertices, instances.per_instance().unwrap()),
                        &self.indices,
                        &self.program,
                        &uniforms,
                        &params).unwrap();
        }
    }
}

use GameContext;
use ecs::components::Appearance;
use ecs::traits::ComponentQuery;
use renderer::interop::RenderUpdate;
use world::World;
use world::traits::Query;

fn tile_in_viewport(viewport: &Viewport, camera: Point, tile: Point) -> bool {
    let min: Point = viewport.min_tile_pos(camera).into();
    let max: Point = viewport.max_tile_pos(camera).into();

    min <= tile && max > tile
}

fn make_sprites(world: &World, viewport: &Viewport) -> Vec<(DrawSprite, (u32, u32))> {
    let mut res = Vec::new();
    let camera = world.flags().camera;
    let start_corner: Point = viewport.min_tile_pos(camera).into();

    let player = match world.player() {
        Some(p) => p,
        None    => return Vec::new(),
    };

    let mut seen = vec![player];
    seen.extend(world.seen_entities(player));

    {
        let mut push_sprite = |pos: Point, appearance: &Appearance| {
            let sprite = DrawSprite {
                kind: appearance.kind.clone()
            };

            // Translate from world tilespace to screen tilespace (where
            // (0, 0) is the upper-left corner)
            let new_pos = pos - start_corner;
            if new_pos.x < 0 || new_pos.y < 0 {
                return;
            }
            assert!(new_pos >= (0, 0), "{}", new_pos);

            res.push((sprite, (new_pos.x as u32, new_pos.y as u32)));
        };

        // First pass: Non-mobs are rendered below mobs.
        for entity in seen.iter() {
            if world.is_mob(*entity) {
                continue;
            }

            if let Some(pos) = world.position(*entity) {
                if !tile_in_viewport(viewport, camera, pos) {
                    continue;
                }

                if let Some(appearance) = world.ecs().appearances.get(*entity) {
                    push_sprite(pos, appearance);
                }
            }
        }

        // Second pass: Draw mobs on top of non-mobs.
        for entity in seen.iter() {
            if world.is_mob(*entity) {
                let pos = world.position(*entity).unwrap();
                let appearance = world.ecs().appearances.get_or_err(*entity);
                push_sprite(pos, appearance);
            }
        }
    }

    res
}

impl RenderUpdate for SpriteMap {
    fn should_update(&self, _context: &GameContext) -> bool {
        true
    }

    fn update(&mut self, context: &GameContext, viewport: &Viewport) {
        let world = &context.state.world;
        self.sprites = make_sprites(world, viewport);
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
