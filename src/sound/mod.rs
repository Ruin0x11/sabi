mod cache;

use self::cache::SoundCache;

make_global!(SOUND, SoundCache, SoundCache::new());

pub fn play(name: &str) {
    instance::with_mut(|sound| sound.play(name));
}
