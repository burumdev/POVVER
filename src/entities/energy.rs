use crate::simulation::SimInt;

pub struct EnergyUnit(SimInt);

impl EnergyUnit {
    pub const fn new(unit: SimInt) -> Self {
        Self(unit)
    }
}
