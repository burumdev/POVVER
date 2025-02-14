use rand::{
    prelude::ThreadRng,
    Rng,
};

mod energy;
mod industries;
mod povver_plant;
mod products;

mod money;
use money::Money;

use crate::{
    simulation::SimFlo,
    utils_random::random_inc_dec_clamp_signed,
};

const MAX_INFLATION: SimFlo = 10000.0;
const MIN_INFLATION: SimFlo = -10.0;
#[derive(Debug)]
pub struct Economy {
    inflation_rate: SimFlo,
    fuel_price: Money,
    rng: ThreadRng,
}

// Constructor
impl Economy {
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();

        Self {
            inflation_rate: rng.gen_range(2.0..10.0),
            fuel_price: Money::new(rng.gen_range(100.00..400.00)),
            rng,
        }
    }
}

impl Economy {
    pub fn update(&mut self) {
        self.inflation_rate = random_inc_dec_clamp_signed(
            &mut self.rng,
            self.inflation_rate,
            0.5,
            0.5,
            MIN_INFLATION,
            MAX_INFLATION,
        );
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
