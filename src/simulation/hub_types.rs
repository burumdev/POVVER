use crate::{
    simulation::SimInt,
};

#[derive(Debug, Clone, PartialEq)]
pub enum MessageEntity {
    Hub,
    PP,
    Factory(SimInt),
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

#[derive(Debug, Clone)]
pub enum DailyJobKind {
    PPFuelCapIncrease,
}

#[derive(Debug, Clone)]
pub struct DailyJob {
    pub kind: DailyJobKind,
    pub delay: SimInt,
    pub day_created: SimInt,
}

#[derive(Debug)]
pub enum PPHubSignal {
    BuyFuel(SimInt),
    IncreaseFuelCapacity,
}

#[derive(Debug, PartialEq)]
pub enum HubPPSignal {
    FuelTransfered,
    FuelCapacityIncreased,
}
