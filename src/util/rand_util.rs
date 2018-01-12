use std::collections::HashSet;
use data::Walkability;

use point::{Point, POINT_ZERO};
use infinigen::ChunkedWorld;
use rand::Rng;
use world::{self, World, Bounds};
use world::traits::*;

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

pub fn random_positions<F: Rng>(world: &World, count: u16, rng: &mut F) -> HashSet<Point> {
    let mut i = 0;
    let mut positions = HashSet::new();
    let mut tries = 0;

    while i < count && tries < 1000 {
        match random_tile(world, rng, |world, point| {
            world.can_walk(point, Walkability::MonstersWalkable)
        }) {
            Some(pos) => {
                if positions.contains(&pos) {
                    tries += 1
                } else {
                    positions.insert(pos);
                    i += 1;
                }
            },
            None => tries += 1,
        }
    }

    positions
}

pub fn random_tile<F: Rng, C>(world: &World, rng: &mut F, callback: C) -> Option<Point>
where
    C: Fn(&World, Point) -> bool,
{
    let mut bound = POINT_ZERO;
    if let Bounds::Bounded(w, h) = *world.terrain().bounds() {
        bound = Point::new(w, h)
    } else {
        return None;
    };

    let mut i = 0;
    while i < 1000 {
        let x = between(0, bound.x, rng);
        let y = between(0, bound.y, rng);

        let point = Point::new(x, y);
        if callback(world, point) {
            return Some(point);
        }
        i += 1;
    }

    None
}
