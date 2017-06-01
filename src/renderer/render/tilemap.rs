use glium;
use glium::backend::Facade;

use point::Direction;
use point::Point;
use renderer::atlas::*;
use renderer::render::{self, Renderable, Vertex, Viewport};

#[derive(Copy, Clone, Debug)]
struct Instance {
    tile_idx: usize,
    map_coord: [i32; 2],
    tex_offset: [f32; 2],
    quadrant: i8,
    autotile: i8,
    autotile_index: i8,
}

implement_vertex!(Instance, map_coord, tex_offset, quadrant, autotile,
                  autotile_index);

#[derive(Debug)]
struct DrawTile {
    kind: &'static str,
    edges: u8,
}

pub struct TileMap {
    tiles: Vec<(DrawTile, Point)>,

    indices: glium::IndexBuffer<u16>,
    vertices: glium::VertexBuffer<Vertex>,
    instances: Vec<glium::VertexBuffer<Instance>>,
    program: glium::Program,

    tile_atlas: TileAtlas,
    valid: bool,
}

fn dir_to_bit(dir: Direction) -> u8 {
    match dir {
        Direction::NE => 0,
        Direction::N  => 1,
        Direction::NW => 2,
        Direction::E  => 3,
        Direction::W  => 4,
        Direction::SE => 5,
        Direction::S  => 6,
        Direction::SW => 7,
    }
}

const QUAD_NW: i8 = 0;
const QUAD_NE: i8 = 1;
const QUAD_SW: i8 = 2;
const QUAD_SE: i8 = 3;

use point::Direction::*;

fn get_autotile_index(edges: u8, quadrant: i8) -> i8 {
    let is_connected = |dir: Direction| (edges & (1 << dir_to_bit(dir))) > 0;

    if !is_connected(N) && !is_connected(W) && !is_connected(E) && !is_connected(S) {
        let ret = match quadrant {
            QUAD_NW => {
                0
            },
            QUAD_NE => {
                1
            },
            QUAD_SW => {
                4
            },
            QUAD_SE => {
                5
            },
            _ => -1,
        };
        return ret;
    }

    // The tiles are in order from the corner inside.
    let lookup_idx = |horiz: Direction, vert: Direction, corner: Direction, tiles: [i8; 4], corner_piece: i8| {
        if !is_connected(horiz) && !is_connected(vert) {
            tiles[0]
        } else if !is_connected(horiz) && is_connected(vert) {
            tiles[1]
        } else if is_connected(horiz) && !is_connected(vert) {
            tiles[2]
        } else {
            if !is_connected(corner) {
                corner_piece
            } else {
                tiles[3]
            }
        }
    };

    match quadrant {
        QUAD_NW => {
            lookup_idx(N, W, NW, [8, 9, 12, 13], 2)
        },
        QUAD_NE => {
            lookup_idx(N, E, NE, [11, 10, 15, 14], 3)
        },
        QUAD_SW => {
            lookup_idx(S, W, SW, [20, 21, 16, 17], 6)
        },
        QUAD_SE => {
            lookup_idx(S, E, SE, [23, 22, 19, 18], 7)
        },
        _ => -1,
    }
}

impl TileMap {
    pub fn new<F: Facade>(display: &F) -> Self {
        let tile_atlas = TileAtlas::from_config(display, "data/tiles.toml");

        let (vertices, indices) = render::make_quad_buffers(display);

        let program = render::load_program(display, "tile.vert", "tile.frag").unwrap();

        let mut tilemap = TileMap {
            tiles: Vec::new(),
            indices: indices,
            vertices: vertices,
            instances: Vec::new(),
            program: program,
            tile_atlas: tile_atlas,
            valid: false,
        };

        tilemap.redraw(display, 0);
        tilemap
    }

    fn make_instances<F>(&mut self, display: &F, msecs: u64)
        where F: glium::backend::Facade {

        let mut instances = Vec::new();

        for pass in 0..self.tile_atlas.passes() {
            let data = self.tiles.iter()
                .filter(|&&(ref tile, _)| {
                    let texture_idx = self.tile_atlas.get_tile_texture_idx(tile.kind);
                    texture_idx == pass
                })
                .flat_map(|&(ref tile, c)| {
                    let mut res = Vec::new();
                    for quadrant in 0..4 {
                        let (x, y) = (c.x, c.y);
                        let (tx, ty) = self.tile_atlas.get_texture_offset(tile.kind, msecs);

                        let autotile_index = get_autotile_index(tile.edges, quadrant);

                        let tile_idx = self.tile_atlas.get_tile_index(&tile.kind);

                        res.push(Instance { tile_idx: tile_idx,
                                            map_coord: [x, y],
                                            tex_offset: [tx, ty],
                                            quadrant: quadrant,
                                            autotile: 1,
                                            autotile_index: autotile_index, });
                    }
                    res
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

impl<'a> Renderable for TileMap {
    fn render<F, S>(&self, _display: &F, target: &mut S, viewport: &Viewport)
        where F: glium::backend::Facade, S: glium::Surface {

        let (proj, scissor) = viewport.main_window();

        for pass in 0..self.tile_atlas.passes() {
            let texture = self.tile_atlas.get_texture(pass);
            let tex_ratio = self.tile_atlas.get_tilemap_tex_ratio(pass);

            let uniforms = uniform! {
                matrix: proj,
                tile_size: [48u32; 2],
                tex: texture.sampled()
                    .wrap_function(glium::uniforms::SamplerWrapFunction::Clamp)
                    .minify_filter(glium::uniforms::MinifySamplerFilter::Nearest)
                    .magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest),
                tex_ratio: tex_ratio,
            };

            let instances = self.instances.get(pass).unwrap();

            let params = glium::DrawParameters {
                blend: glium::Blend::alpha_blending(),
                scissor: Some(scissor),
                .. Default::default()
            };

            target.draw((&self.vertices, instances.per_instance().unwrap()),
                        &self.indices,
                        &self.program,
                        &uniforms,
                        &params).unwrap();
        }
    }
}

use graphics::cell::CellType;
use world::EcsWorld;
use world::traits::{Query, WorldQuery};
use GameContext;
use renderer::interop::RenderUpdate;

fn get_neighboring_edges(world: &EcsWorld, pos: Point, cell_type: CellType) -> u8 {
    let mut res: u8 = 0;
    for dir in Direction::iter8() {
        let new_pos = pos + *dir;
        let same_type = world.cell_const(&new_pos).map_or(false, |c| c.type_ == cell_type);
        if same_type {
            res |= 1 << dir_to_bit(*dir);
        }
    }
    res
}

fn make_map(world: &EcsWorld, viewport: &Viewport) -> Vec<(DrawTile, Point)> {
    let mut res = Vec::new();
    let camera = world.flags().camera;
    let start_corner = viewport.camera_tile_pos(camera).into();
    world.with_cells(start_corner, Viewport::renderable_area().into(), |pos, &cell| {
        let tile = DrawTile {
            kind: cell.glyph(),
            edges: get_neighboring_edges(world, pos, cell.type_),
        };
        res.push((tile, pos - start_corner));

        if let Some(feature) = cell.feature {
            let feature_tile = DrawTile {
                kind: feature.glyph(),
                edges: 0,
            };
            res.push((feature_tile, pos - start_corner));
        }
    });
    res
}

impl RenderUpdate for TileMap {
    fn should_update(&self, _context: &GameContext) -> bool {
        true
    }

    fn update(&mut self, context: &GameContext, viewport: &Viewport) {
        let ref world = context.state.world;
        self.tiles = make_map(world, viewport);
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

