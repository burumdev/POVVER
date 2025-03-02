use crate::{
    economy::economy_types::{EnergyUnit, Money},
    simulation::SimInt,
};

#[derive(Debug)]
pub struct EnergyOffer {
    price: Money,
    amount: EnergyUnit,
}

#[derive(Debug)]
pub enum HourlyJob {
    PPBoughtFuel(SimInt),
}

#[derive(Debug)]
pub enum PovverPlantSignal {
    BuyFuel(SimInt),
}

#[derive(Debug)]
pub enum BroadcastSignal {
    SellingEnergy(EnergyOffer),
}

#[derive(Debug)]
pub enum FactorySignal {
    BuyEnergy(EnergyUnit),
}
