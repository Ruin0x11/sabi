/// A collection of stats.
pub struct StatsInit {
    pub hp: u32,
    pub strength: u32,
    pub defense: u32,
}

pub struct Stats(StatsInit);

impl Default for Stats {
    fn default() -> Self {
        Stats(StatsInit {
            hp: 1,
            strength: 1,
            defense: 1,
        })
    }
}

impl Stats {
    // TODO: Builder
    // It doesn't make sense to have 0 hp, so the number has to be checked. But
    // checking means passing in arguments to 'new', and the number of arguments
    // could grow largely.
    pub fn new(init: StatsInit) -> Self {
        assert!(init.hp > 0);
        Stats(init)
    }

    pub fn max_hp(&self) -> u32 {
        self.0.hp
    }

    pub fn max_strength(&self) -> u32 {
        self.0.strength
    }

    pub fn max_defense(&self) -> u32 {
        self.0.defense
    }

    pub fn hp(&self) -> u32 {
        self.0.hp
    }

    pub fn strength(&self) -> u32 {
        self.0.strength
    }

    pub fn defense(&self) -> u32 {
        self.0.defense
    }
}
