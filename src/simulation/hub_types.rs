use crate::{
    economy::economy_types::{EnergyUnit, Money},
    simulation::SimInt,
};

#[derive(Debug)]
pub struct EnergyOffer {
    price: Money,
    amount: EnergyUnit,
}

#[derive(Debug, Clone)]
pub enum HourlyJobKind {
    PPBoughtFuel(SimInt),
}

#[derive(Debug, Clone)]
pub struct HourlyJob {
    pub kind: HourlyJobKind,
    pub delay: SimInt,
    pub hour_created: SimInt,
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
