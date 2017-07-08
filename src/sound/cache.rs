use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;

use rodio::{self, Source};
use rodio::buffer::SamplesBuffer;

struct Sound {
    channels: u16,
    samples_rate: u32,
    samples: Vec<f32>,
}

impl Sound {
    pub fn to_buffer(&self) -> SamplesBuffer<f32> {
        SamplesBuffer::new(self.channels, self.samples_rate, self.samples.clone())
    }
}

pub struct SoundCache {
    cache: HashMap<String, Sound>,
    endpoint: rodio::Endpoint,
}

impl SoundCache {
    pub fn new() -> Self {
        SoundCache {
            cache: HashMap::new(),
            endpoint: rodio::get_default_endpoint().unwrap(),
        }
    }

    fn get_path(name: &str) -> String {
        format!("data/sound/{}.wav", name)
    }

    fn cache_sound(&mut self, name: &str) {
        let path = SoundCache::get_path(name);
        let file = File::open(&path).unwrap();
        let source = rodio::Decoder::new(BufReader::new(file)).unwrap();

        let sample = Sound {
            channels: source.channels(),
            samples_rate: source.samples_rate(),
            samples: source.convert_samples().collect::<Vec<f32>>(),
        };

        self.cache.insert(name.to_string(), sample);
    }

    pub fn play(&mut self, name: &str) {
        if !self.cache.contains_key(name) {
            self.cache_sound(name);
        }

        let sound = self.cache.get(name).unwrap();
        rodio::play_raw(&self.endpoint, sound.to_buffer().amplify(0.2));
    }
}
