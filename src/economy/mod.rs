use std::{
    sync::{Arc, RwLock}
};
use rand::{prelude::ThreadRng, random, Rng};

mod industries;
mod products;

pub mod economy_types;
use economy_types::*;
pub mod factory;
pub mod povver_plant;

use crate::{
    utils_random::{one_chance_in_many, random_inc_dec_clamp_signed},
    utils_traits::{Flippable, Percentage},
    app_state::EconomyStateData,
};

#[derive(Debug)]
pub struct Economy {
    state: Arc<RwLock<EconomyStateData>>,
    rng: ThreadRng,
}

// Constructor
impl Economy {
    pub fn new() -> (Self, Arc<RwLock<EconomyStateData>>) {
        let mut rng = rand::thread_rng();

        let inflation_direction = if random() { UpDown::Up } else { UpDown::Down };
        let state = Arc::new(RwLock::new(EconomyStateData {
            inflation_rate: rng.gen_range(2.0..10.0),
            inflation_direction,
            fuel_price: Money::new(rng.gen_range(100.00..200.00)),
        }));

        (
            Self {
                state: Arc::clone(&state),
                rng,
            },
            state,
        )
    }
}

impl Economy {
    pub fn update_macroeconomics(&mut self) {
        let mut inflation_direction = self.state.read().unwrap().inflation_direction.clone();

        if one_chance_in_many(&mut self.rng,33) {
            inflation_direction = inflation_direction.flip();
            self.state.write().unwrap().inflation_direction.flip();
        }

        let inflation_low_end = if inflation_direction == UpDown::Down { 1.0 } else { 0.2 };
        let inflation_high_end = if inflation_direction == UpDown::Up { 1.0 } else { 0.2 };
        let inflation_rate = {
            let mut inflation_rate = self.state.read().unwrap().inflation_rate;
            inflation_rate = random_inc_dec_clamp_signed(
                &mut self.rng,
                inflation_rate,
                inflation_low_end,
                inflation_high_end,
                INFLATION_MIN,
                INFLATION_MAX,
            );
            self.state.write().unwrap().inflation_rate = inflation_rate;

            inflation_rate
        };

        let fuel_price_low_end = FUEL_PRICE_MODIFIER - (inflation_rate.as_factor() * FUEL_PRICE_MODIFIER);
        let fuel_price_high_end = FUEL_PRICE_MODIFIER + (inflation_rate.as_factor() * FUEL_PRICE_MODIFIER);
        let fuel_price = self.state.read().unwrap().fuel_price;
        self.state.write().unwrap().fuel_price.set(random_inc_dec_clamp_signed(
            &mut self.rng,
            fuel_price.val(),
            fuel_price_low_end,
            fuel_price_high_end,
            FUEL_PRICE_MIN,
            FUEL_PRICE_MAX,
        ));
    }
}
