use std::fmt::{self, Display};

use rand::{self, Rng};
use rand::distributions::{Range, IndependentSample};

use actor::Actor;

struct Dice {
    rolls: u32,
    sides: u32,
    bonus: u32,
}

impl Dice {
    pub fn new(rolls: u32, sides: u32, bonus: u32) -> Self {
        assert!(rolls > 0);
        assert!(sides > 0);
        Dice {
            rolls: rolls,
            sides: sides,
            bonus: bonus,
        }
    }

    pub fn roll(&self) -> u32 {
        let mut rng = rand::thread_rng();
        let lower = self.bonus;
        let upper = lower + self.sides + 1;
        let range = Range::new(lower, upper);
        let mut result = 0;
        for _ in 0..self.rolls {
            result += range.ind_sample(&mut rng);
        }
        result
    }
}

impl Display for Dice {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<{}d{}", self.rolls, self.sides)?;
        if self.bonus > 0 {
            write!(f, " + {}", self.bonus)?;
        }
        write!(f, ">")
    }
}

pub fn calculate_delay(actor: &Actor, action_cost: u32) -> i32 {
    (100*action_cost / actor.speed) as i32
}

pub fn check_evasion(attacker: &Actor, defender: &Actor) -> bool {
    false
}

pub fn calculate_damage(attacker: &Actor, defender: &Actor) -> u32 {
    let dice = Dice::new(2, 4, 4);
    debug!(attacker.logger, "{}: attacking {} with {}", attacker, defender, dice);

    dice.roll()
}
