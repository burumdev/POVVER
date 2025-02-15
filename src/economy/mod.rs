use rand::{prelude::ThreadRng, random, Rng};

mod industries;
mod povver_plant;
mod products;

mod economy_types;
use economy_types::*;

use crate::{
    simulation::SimFlo,
    utils_random::{random_inc_dec_clamp_signed, one_chance_in_many},
    utils_traits::Flippable,
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
    pub fn update(&mut self) {
        if one_chance_in_many(&mut self.rng,33) {
            self.inflation_direction.flip();
        }
        let inflation_low_end = if self.inflation_direction == UpDown::Down { 5.0 } else { 0.2 };
        let inflation_high_end = if self.inflation_direction == UpDown::Up { 5.0 } else { 0.2 };
        self.inflation_rate = random_inc_dec_clamp_signed(
            &mut self.rng,
            self.inflation_rate,
            inflation_low_end,
            inflation_high_end,
            INFLATION_MIN,
            INFLATION_MAX,
        );

        let fuel_price_low_end =
        self.fuel_price.set_amount(random_inc_dec_clamp_signed(
            &mut self.rng,
            self.fuel_price.get(),
            35.00,
            35.00,
            100.0,
            40000.0,
        ));
    }
}
