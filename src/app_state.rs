use std::sync::{Arc, Mutex, RwLock};
use crate::{
    environment::{TheSun, WindSpeed, months::Month},
    simulation::{SimFlo, SimInt},
    ui_controller::{Cloud, Date, WindDirection},
    economy::economy_types::{Money, EnergyUnit},
};

#[derive(Debug)]
pub struct TimerStateData {
    pub date: Date,
    pub month_data: &'static Month,
}

#[derive(Debug)]
pub struct EnvStateData {
    pub clouds: Vec<Cloud>,
    pub wind_speed: WindSpeed,
    pub wind_direction: WindDirection,
    pub the_sun: TheSun,
    pub sunshine_reduction: SimFlo,
}

#[derive(Debug, Clone)]
pub enum Misc {
    IsPaused(bool),
    SpeedIndex(usize),
}

#[derive(Debug, Clone)]
pub struct MiscStateData {
    pub is_paused: bool,
    pub speed_index: usize,
}

pub struct PovverPlantStateData {
    pub fuel: SimInt,
    pub fuel_capacity: SimInt,
    pub production_capacity: EnergyUnit,
    pub balance: Money,
}

pub struct FactoryStateData {
    balance: Money,
}

pub struct EconomyState {
    pub povver_plant: Arc<RwLock<PovverPlantStateData>>,
    pub factories: Arc<Vec<RwLock<FactoryStateData>>>,
}

pub struct AppState {
    pub timer: Arc<RwLock<TimerStateData>>,
    pub env: Arc<RwLock<EnvStateData>>,
    pub misc: Arc<Mutex<MiscStateData>>,
    pub is_misc_updated: bool,
}

#[derive(Debug)]
pub struct StatePayload {
    pub timer: Arc<RwLock<TimerStateData>>,
    pub env: Arc<RwLock<EnvStateData>>,
    pub misc: Arc<Mutex<MiscStateData>>,
}

impl AppState {
    pub fn new(
        timer: Arc<RwLock<TimerStateData>>,
        env: Arc<RwLock<EnvStateData>>,
        misc: Arc<Mutex<MiscStateData>>
    ) -> Self {

        Self {
            timer,
            env,
            misc,
            is_misc_updated: true,
        }
    }
}

impl AppState {
    pub fn get_state_payload(&self) -> Arc<StatePayload> {
        Arc::new(StatePayload {
            timer: Arc::clone(&self.timer),
            env: Arc::clone(&self.env),
            misc: Arc::clone(&self.misc),
        })
    }

    pub fn set_misc(&mut self, misc: Misc) {
        match misc {
            Misc::IsPaused(val) => {
                self.misc.lock().unwrap().is_paused = val;
            },
            Misc::SpeedIndex(val) => {
                self.misc.lock().unwrap().speed_index = val;
            },
        }

        self.is_misc_updated = true;
    }

    pub fn get_misc_state_updates(&mut self) -> Option<MiscStateData> {
        if self.is_misc_updated {
            self.is_misc_updated = false;
            Some(self.misc.lock().unwrap().clone())
        } else {
            None
        }
    }

    pub fn is_misc_updated(&self) -> bool {
        self.is_misc_updated
    }
}