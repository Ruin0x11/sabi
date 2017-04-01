pub mod archetype;
pub mod formulas;
pub mod properties;

const STAT_HP: usize = 0;
const STAT_STRENGTH: usize = 1;
const STAT_DEFENSE: usize = 2;
const NUM_STATS: usize = 3;

/// A collection of stats.
pub struct Stats {
    // The number of stats is always known exactly.

    max_stats: [i32; NUM_STATS],
}

impl Stats {
    pub fn hp(&self) -> i32 {
        self.max_stats[STAT_HP]
    }

    pub fn strength(&self) -> i32 {
        self.max_stats[STAT_STRENGTH]
    }

    pub fn defense(&self) -> i32 {
        self.max_stats[STAT_DEFENSE]
    }
}
