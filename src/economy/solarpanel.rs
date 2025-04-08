use crate::{
    simulation::{
        SimInt,
        timer::TimerEvent,
        sim_constants::*
    },
    utils_traits::AsFactor,
    environment::SunBrightness,
};

#[derive(Debug, Clone)]
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

    pub fn produce_energy(&mut self, timer_event: &TimerEvent, sunshine: SunBrightness) -> SimInt {
        if timer_event.at_least_month() {
            self.age += 1;
            if self.age >= SOLAR_PANEL_MAX_AGE {
                self.is_defunct = true;
            }
        }

        if self.is_defunct {
            return 0;
        }

        let mut energy = sunshine.val() as SimInt;
        energy -= self.age * SOLAR_PANEL_AGE_MODIFIER;

        energy
    }
}