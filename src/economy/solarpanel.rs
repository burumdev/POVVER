use crate::{
    economy::economy_types::EnergyUnit,
    simulation::{
        SimInt,
        timer::TimerEvent,
        sim_constants::*
    },
    utils_traits::AsFactor,
    environment::SunBrightness,
};

#[derive(Debug)]
pub struct SolarPanel {
    age: SimInt,
    is_defunct: bool,
}

impl SolarPanel {
    pub fn new() -> Self {
        Self {
            age: 0,
            is_defunct: false,
        }
    }

    pub fn produce_energy(&mut self, timer_event: TimerEvent, sunshine: SunBrightness) -> EnergyUnit {
        if timer_event.at_least_year() {
            self.age += 1;
            if self.age >= SOLAR_PANEL_MAX_AGE {
                self.is_defunct = true;
            }
        }

        if self.is_defunct {
            return EnergyUnit::new(0);
        }

        let mut e = sunshine.val() as SimInt;
        e -= self.age * SOLAR_PANEL_AGE_MODIFIER;

        EnergyUnit::new(e)
    }
}