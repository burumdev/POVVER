use std::sync::{Arc, RwLock};
use crate::{
    app_state::FactoryStateData,
    economy::{
        economy_types::{EnergyUnit, Money},
        industries::Industry,
        products::Product,
    },
    simulation::sim_constants::{FACTORY_INIT_MONEY, FACTORY_MAX_SOLAR_PANELS},
};

pub const TEST_INDUSTRIES: [Industry; 5] = [
    Industry::SEMICONDUCTORS,
    Industry::COSMETICS,
    Industry::PROCESSED_FOODS,
    Industry::ARMS,
    Industry::BANK,
];

pub fn get_test_factories() -> Vec<Arc<RwLock<FactoryStateData>>> {
    let mut factory_states = Vec::new();
    for (id, industry) in TEST_INDUSTRIES.iter().enumerate() {
        let industry_products = Product::by_industry(&industry);
        let cheapest_rnd_product = industry_products
            .iter()
            .min_by(|prod_a, prod_b| prod_a.rnd_cost.total_cmp(&prod_b.rnd_cost)).unwrap();

        let product_portfolio = vec![*cheapest_rnd_product];

        factory_states.push(
            Arc::new(
                RwLock::new(
                    FactoryStateData {
                        balance: Money::new(FACTORY_INIT_MONEY - product_portfolio[0].rnd_cost),
                        available_energy: EnergyUnit::default(),
                        product_stocks: Vec::new(),
                        solarpanels: Vec::with_capacity(FACTORY_MAX_SOLAR_PANELS),
                        industry: industry.clone(),
                        product_portfolio,
                        id,
                        is_bankrupt: false,
                        is_awaiting_solarpanels: false,
                    }
                )
            )
        )
    }

    factory_states
}