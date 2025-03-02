use crate::{
    economy::economy_types::{EnergyUnit, Money},
    simulation::SimInt,
};

pub struct EnergyOffer {
    price: Money,
    amount: EnergyUnit,
}

pub enum HourlyJobs {
    PPBoughtFuel(SimInt),
}

pub enum PovverPlantSignals {
    BuyFuel(SimInt),
}

pub enum BroadcastSignals {
    FuelPrice(Money),
    SellingEnergy(EnergyOffer),
}

pub enum FactorySignals {
    BuyEnergy(EnergyUnit),
}
