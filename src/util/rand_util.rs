use rand::Rng;

pub fn zero_to<F: Rng>(n: u32, rng: &mut F) -> u32 {
    rng.gen_range(0, n)
}

pub fn between<F: Rng>(a: i32, b: i32, rng: &mut F) -> i32 {
    rng.gen_range(a, b)
}

pub fn chance<F: Rng>(n: f32, rng: &mut F) -> bool {
    rng.next_f32() < n
}

pub fn coinflip<F: Rng>(rng: &mut F) -> bool {
    rng.gen()
}
