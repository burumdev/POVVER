use crate::{
    simulation::SimInt,
};
use crate::economy::economy_types::EnergyUnit;


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

