use std::{
    sync::{Arc, RwLock}
};
use rand::{prelude::ThreadRng, random, Rng};

pub mod industries;
pub mod products;

pub mod economy_types;
use economy_types::*;

pub mod economy_constants;
use economy_constants::*;

pub mod factory;
pub mod povver_plant;
pub mod solarpanel;

use crate::{
    utils_random::{one_chance_in_many, random_inc_dec_clamp_signed, random_inc_dec_clamp_unsigned},
    utils_traits::{Flippable, AsFactor},
    utils_data::SlidingWindow,
    app_state::EconomyStateData,
    economy::products::PRODUCTS,
    simulation::{SimFlo, Percentage},
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
            fuel_price: Money::new(200.0),
            product_demands: Vec::new(),
            past_25_product_demands: SlidingWindow::new(25),
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

    pub fn maybe_new_product_demands(&mut self) {
        let inflation = self.state.read().unwrap().inflation_rate;
        let inflation_hundred = if inflation > 0.0 {
            inflation * 2.0
        } else {
            -1.0
        };

        for product in PRODUCTS {
            let min_percent = product.demand_info.min_percentage;
            if inflation_hundred < min_percent && inflation_hundred != -1.0 {
                let mut bonus = 0.0;
                self.state.read().unwrap().past_25_product_demands
                    .iter()
                    .filter(|demand| demand.product == product)
                    .for_each(|demand| {
                        match demand.demand_meet_percent.val() {
                            // If demand is overly met in the past. There's a negative bonus
                            mp if mp <= 100.0 && mp >= 90.0 => bonus += -6.0,
                            mp if mp < 90.0 && mp >= 80.0 => bonus += 0.0,
                            // If demand was met just right in the past, we get a nice bonus,
                            // This means the consumers are eager to get more of the products
                            // because there's both availability, price advantage and product is famous.
                            mp if mp < 80.0 && mp >= 60.0 => bonus += 8.0,
                            mp if mp < 60.0 && mp >= 50.0 => bonus += 5.0,
                            // If factories couldn't meet the demand or worse yet
                            // could only meet a fraction of it, then there's frustration
                            // among the consumers, who'll be cold to the product.
                            mp if mp < 50.0 && mp >= 30.0 => bonus += 0.0,
                            mp if mp < 30.0 && mp >= 10.0 => bonus += -6.0,
                            mp if mp < 10.0 => bonus += -10.0,
                            // This should be unreachable because Percentage should be clamped between 0.0 and 100.0
                            _ => unreachable!()

                        }
                    });

                // Let's create a new demand for this product
                // With bonus added.
                self.state.write().unwrap().product_demands.push(
                    ProductDemand::new(product, Percentage::new(min_percent + bonus))
                );
            // If inflation is negative (deflation) we still have a
            // chance for a new demand with minimum percentage.
            // In deflationary times, consumers expect product prices to
            // decrease even further in the future, so they postpone consuming.
            } else if inflation_hundred == -1.0 {
                let chance = random_inc_dec_clamp_unsigned(
                    &mut self.rng,
                    24u32,
                    8,
                    8,
                    16,
                    32
                );
                // If minimum percentage of demand is high enough though this should equate to
                // a high chance of new demand being created. Products like bullets or
                // pregnancy tests will always have a demand, albeit the lowest percentage
                // in deflationary times.
                if one_chance_in_many(&mut self.rng, chance - (min_percent.as_factor() * chance as SimFlo) as u32) {
                    self.state.write().unwrap().product_demands.push(
                        ProductDemand::new(product, Percentage::new(min_percent))
                    )
                }
            }
        }
    }

    pub fn update_product_demands(&self) {
        let mut old_demands = Vec::new();
        let mut demands = self.state.read().unwrap().product_demands.clone();
        demands.retain_mut(|demand| {
            if demand.demand_meet_percent.val() == 100.0 {
                old_demands.push(demand.clone());

                return false;
            }
            demand.age += 1;
            let demand_timeline = &demand.product.demand_info.demand_timeline;
            match demand.age {
                age if age >= demand_timeline.deadline || demand.percent < 0.01 => {
                    old_demands.push(demand.clone());

                    return false;
                }
                age if age >= demand_timeline.dec_three_quarters => {
                    demand.percent *= 0.25;
                }
                age if age >= demand_timeline.dec_half => {
                    demand.percent *= 0.50;
                }
                age if age >= demand_timeline.dec_quarter => {
                    demand.percent *= 0.75;
                }
                age if age >= demand_timeline.inc_quarter => {
                    demand.percent *= 1.25;
                }
                _ => ()
            }

            true
        });

        self.state.write().unwrap().product_demands = demands;

        for demand in old_demands.drain(..) {
            self.state.write().unwrap().past_25_product_demands.add(demand);
        }
    }
}
