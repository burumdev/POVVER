use crate::economy::economy_types::Money;
use crate::economy::industries::Industry;
use crate::economy::products::{Product, PRODUCTS};

pub const FACTORY_INIT_MONEY: Money = Money::new(10000.0);

pub struct Factory {
    industry: Industry,
    products: Vec<Product>,
    balance: Money,
}

impl Factory {
    pub fn new(industry: Industry) -> Self {
        Self {
            industry,
            products: Vec::with_capacity(2),
            balance: FACTORY_INIT_MONEY,
        }
    }
}