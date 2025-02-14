use crate::economy::energy::EnergyUnit;
use crate::simulation::SimInt;

pub struct PovverPlant {
    fuel: SimInt,
    production_capacity: EnergyUnit,
}
