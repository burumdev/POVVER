use rand::{prelude::ThreadRng, random, Rng};

mod industries;
mod povver_plant;
mod products;

mod economy_types;
use economy_types::*;

use crate::{
    simulation::SimFlo,
    utils_random::{random_inc_dec_clamp_signed, one_chance_in_many},
    utils_traits::{Flippable, Percentage},
};

#[derive(Debug)]
pub struct Economy {
    inflation_rate: SimFlo,
    inflation_direction: UpDown,
    fuel_price: Money,
    rng: ThreadRng,
}

// Constructor
impl Economy {
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();

        let inflation_direction = if random() { UpDown::Up } else { UpDown::Down };
        Self {
            inflation_rate: rng.gen_range(2.0..10.0),
            inflation_direction,
            fuel_price: Money::new(rng.gen_range(100.00..400.00)),
            rng,
        }
    }
}

impl Economy {
    pub fn update_macroeconomics(&mut self) {
        if one_chance_in_many(&mut self.rng,33) {
            self.inflation_direction.flip();
        }
        let inflation_low_end = if self.inflation_direction == UpDown::Down { 1.0 } else { 0.2 };
        let inflation_high_end = if self.inflation_direction == UpDown::Up { 1.0 } else { 0.2 };
        self.inflation_rate = random_inc_dec_clamp_signed(
            &mut self.rng,
            self.inflation_rate,
            inflation_low_end,
            inflation_high_end,
            INFLATION_MIN,
            INFLATION_MAX,
        );

        let fuel_price_low_end = FUEL_PRICE_MODIFIER - (self.inflation_rate.as_factor() * FUEL_PRICE_MODIFIER);
        let fuel_price_high_end = FUEL_PRICE_MODIFIER + (self.inflation_rate.as_factor() * FUEL_PRICE_MODIFIER);
        self.fuel_price.set(random_inc_dec_clamp_signed(
            &mut self.rng,
            self.fuel_price.val(),
            fuel_price_low_end,
            fuel_price_high_end,
            FUEL_PRICE_MIN,
            FUEL_PRICE_MAX,
        ));
    }
}
