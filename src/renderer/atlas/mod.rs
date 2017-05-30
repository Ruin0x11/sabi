use std::collections::HashMap;
use std::fs::{self, File};
use std::path::Path;

use glium;
use glium::backend::Facade;
use image::{self, DynamicImage, GenericImage, Rgba};
use texture_packer::Rect;
use texture_packer::SkylinePacker;
use texture_packer::{TexturePacker, TexturePackerConfig};
use texture_packer::importer::ImageImporter;
use texture_packer::exporter::ImageExporter;

mod config;
pub mod font;
pub mod texture_atlas;

use self::config::TileAtlasConfig;

pub type TileOffset = (u32, u32);

pub type Texture2d = glium::texture::CompressedSrgbTexture2d;

type AnimFrames = u64;
type AnimMillisDelay = u64;
#[derive(Serialize, Deserialize, Clone)]
pub enum TileKind {
    Static,
    Animated(AnimFrames, AnimMillisDelay),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AtlasRect {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

impl From<Rect> for AtlasRect {
    fn from(rect: Rect) -> AtlasRect {
        AtlasRect {
            x: rect.x,
            y: rect.y,
            w: rect.w,
            h: rect.h,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AtlasTile {
    pub offset: TileOffset,
    pub is_autotile: bool,
    pub tile_kind: TileKind,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AtlasFrame {
    tile_size: (u32, u32),
    texture_idx: usize,
    rect: AtlasRect,
    offsets: HashMap<String, AtlasTile>,
}

impl AtlasFrame {
    pub fn new(texture_idx: usize, rect: Rect, tile_size: (u32, u32)) -> Self {
        AtlasFrame {
            tile_size: tile_size,
            texture_idx: texture_idx,
            rect: AtlasRect::from(rect),
            offsets: HashMap::new(),
        }
    }
}

pub type TilePacker<'a> = TexturePacker<'a, DynamicImage, SkylinePacker<Rgba<u8>>>;

pub struct TileAtlas {
    config: TileAtlasConfig,
    textures: Vec<Texture2d>,
}

pub struct TileAtlasBuilder<'a> {
    locations: HashMap<String, String>,
    frames: HashMap<String, AtlasFrame>,
    packers: Vec<TilePacker<'a>>,
    pub file_hash: String,
}

impl <'a> TileAtlasBuilder<'a> {
    pub fn new() -> Self {
        let mut builder = TileAtlasBuilder {
            locations: HashMap::new(),
            frames: HashMap::new(),
            packers: Vec::new(),
            file_hash: String::new(),
        };
        builder.add_packer();
        builder
    }

    pub fn add_tile(&mut self, path_str: &str, index: String, tile_data: AtlasTile) {
        let key = path_str.to_string();
        assert!(self.frames.contains_key(&path_str.to_string()));

        {
            let mut frame = self.frames.get_mut(&key).unwrap();
            assert!(!frame.offsets.contains_key(&index));
            frame.offsets.insert(index.clone(), tile_data);
            self.locations.insert(index, key);
        }
    }

    pub fn add_frame(&mut self, path_string: &str, tile_size: (u32, u32)) {
        if self.frames.contains_key(path_string) {
            return;
        }

        let path = Path::new(&path_string);
        let texture = ImageImporter::import_from_file(&path).unwrap();

        for (idx, packer) in self.packers.iter_mut().enumerate() {
            if packer.can_pack(&texture) {
                packer.pack_own(path_string.to_string(), texture).unwrap();
                let rect = packer.get_frame(&path_string).unwrap().frame.clone();
                self.frames.insert(path_string.to_string(), AtlasFrame::new(idx, rect, tile_size));
                // cannot return self here, since self already borrowed, so
                // cannot use builder pattern.
                return;
            }
        }

        self.add_packer();

        {
            // complains that borrow doesn't last long enough
            // len mut packer = self.newest_packer_mut();

            let packer_idx = self.packers.len() - 1;
            let mut packer = self.packers.get_mut(packer_idx).unwrap();
            packer.pack_own(path_string.to_string(), texture).unwrap();
            let rect = packer.get_frame(&path_string).unwrap().frame.clone();
            self.frames.insert(path_string.to_string(), AtlasFrame::new(packer_idx, rect, tile_size));
        }
    }

    fn add_packer(&mut self) {
        let config = TexturePackerConfig {
            max_width: 2048,
            max_height: 2048,
            allow_rotation: false,
            texture_outlines: false,
            trim: false,
            texture_padding: 0,
            ..Default::default()
        };

        self.packers.push(TexturePacker::new_skyline(config));
    }

    pub fn build<F: Facade>(&self, display: &F, packed_tex_folder: &str) -> TileAtlas {
        let mut textures = Vec::new();

        let packed_folder_path = config::get_config_cache_path(packed_tex_folder);

        if Path::exists(packed_folder_path.as_path()) {
            fs::remove_dir_all(packed_folder_path.as_path()).unwrap();
        }

        fs::create_dir_all(packed_folder_path.as_path()).unwrap();

        for (idx, packer) in self.packers.iter().enumerate() {
            let image = ImageExporter::export(packer).unwrap();

            let mut file_path = packed_folder_path.clone();
            file_path.push(&format!("{}.png", idx));

            let mut file = File::create(file_path).unwrap();

            image.save(&mut file, image::PNG).unwrap();
            textures.push(make_texture(display, image));
        }

        println!("Saved {}", packed_tex_folder);

        let config = TileAtlasConfig {
            locations: self.locations.clone(),
            frames: self.frames.clone(),
            file_hash: self.file_hash.clone(),
        };

        config::write_tile_atlas_config(&config, packed_tex_folder);

        TileAtlas {
            config: config,
            textures: textures,
        }
    }
}

impl TileAtlas {
    pub fn new(config: TileAtlasConfig, textures: Vec<Texture2d>) -> Self {
        TileAtlas {
            config: config,
            textures: textures,
        }
    }

    fn get_frame(&self, tile_type: &str) -> &AtlasFrame {
        let tex_name = self.config.locations.get(tile_type).unwrap();
        self.config.frames.get(tex_name).unwrap()
    }

    pub fn get_tile_texture_idx(&self, tile_type: &str) -> usize {
        self.get_frame(tile_type).texture_idx
    }


    pub fn get_tilemap_tex_ratio(&self, texture_idx: usize) -> [f32; 2] {
        let dimensions = self.textures.get(texture_idx).unwrap().dimensions();

        let cols: u32 = dimensions.0 / 24;
        let rows: u32 = dimensions.1 / 24;
        [1.0 / cols as f32, 1.0 / rows as f32]
    }

    pub fn get_sprite_tex_ratio(&self, tile_type: &str) -> [f32; 2] {
        let frame = self.get_frame(tile_type);
        let (mut sx, mut sy) = frame.tile_size;

        if frame.offsets.get(tile_type).unwrap().is_autotile {
            // divide the autotile into 24x24 from 48x48
            sx /= 2;
            sy /= 2;
        }

        let texture_idx = self.get_frame(tile_type).texture_idx;
        let dimensions = self.textures.get(texture_idx).unwrap().dimensions();

        let cols: f32 = dimensions.0 as f32 / sx as f32;
        let rows: f32 = dimensions.1 as f32 / sy as f32;
        [1.0 / cols, 1.0 / rows]
    }

    pub fn get_tile_texture_size(&self, tile_type: &str) -> (u32, u32) {
        self.get_frame(tile_type).tile_size
    }

    pub fn get_texture_offset(&self, tile_type: &str, msecs: u64) -> (f32, f32) {
        let frame = self.get_frame(tile_type);
        let tile = frame.offsets.get(tile_type).unwrap();

        let get_tex_coords = |index: (u32, u32)| {
            let tex_ratio = self.get_sprite_tex_ratio(tile_type);
            let mut add_offset = get_add_offset(&frame.rect, &frame.tile_size);

            match tile.tile_kind {
                TileKind::Static => (),
                TileKind::Animated(frame_count, delay) => {
                    let current_frame = msecs / delay;
                    let mut x_index_offset = current_frame % frame_count;

                    if tile.is_autotile {
                        x_index_offset *= 2;
                    }

                    add_offset.0 += x_index_offset as u32;
                }
            }

            let mut ratio = 1;

            if tile.is_autotile {
                ratio = 2;
            }

            let tx = ((index.0 + add_offset.0) * ratio) as f32 * tex_ratio[0];
            let ty = ((index.1 + add_offset.1) * ratio) as f32 * tex_ratio[1];

            (tx, ty)
        };

        get_tex_coords(tile.offset)
    }

    pub fn get_texture(&self, idx: usize) -> &Texture2d {
        self.textures.get(idx).unwrap()
    }

    pub fn passes(&self) -> usize {
        self.textures.len()
    }
}

fn get_add_offset(rect: &AtlasRect, tile_size: &(u32, u32)) -> (u32, u32) {
    let ceil = |a, b| (a + b - 1) / b;
    let cols: u32 = ceil(rect.x, tile_size.0);
    let rows: u32 = ceil(rect.y, tile_size.1);
    (cols, rows)
}

pub fn make_texture<F: Facade>(display: &F, image: DynamicImage) -> Texture2d {
    let dimensions = image.dimensions();
    let image = glium::texture::RawImage2d::from_raw_rgba_reversed(image.to_rgba().into_raw(), dimensions);
    Texture2d::new(display, image).unwrap()
}
