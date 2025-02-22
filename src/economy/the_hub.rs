use crate::economy::factory::Factory;
use crate::economy::povver_plant::PovverPlant;

pub struct TheHub {
    povver_plant: PovverPlant,
    factories: Vec<Factory>,
}

impl TheHub {
    pub fn new() -> Self {
        Self {
            povver_plant: PovverPlant::new(),
            factories: Vec::new(),
        }
    }
}