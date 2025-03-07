use std::sync::{Arc, Mutex, RwLock};
use crate::{
    environment::{TheSun, WindSpeed, months::Month},
    simulation::{SimFlo, SimInt},
    ui_controller::{Cloud, Date, WindDirection},
    economy::economy_types::{Money, EnergyUnit, UpDown, ProductDemand},
    utils_data::ReadOnlyRwLock,
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

#[derive(Debug)]
pub struct PovverPlantStateData {
    pub fuel: SimInt,
    pub fuel_capacity: SimInt,
    pub production_capacity: EnergyUnit,
    pub balance: Money,
    pub is_awaiting_fuel: bool,
    pub is_bankrupt: bool,
}

#[derive(Debug)]
pub struct FactoryStateData {
    balance: Money,
}

#[derive(Debug)]
pub struct EconomyStateData {
    pub inflation_rate: SimFlo,
    pub inflation_direction: UpDown,
    pub fuel_price: Money,
    pub product_demands: Vec<ProductDemand>,
}

pub struct HubState {
    pub povver_plant: Arc<RwLock<PovverPlantStateData>>,
    pub factories: Arc<RwLock<Vec<FactoryStateData>>>,
}

pub struct AppState {
    pub timer: Arc<RwLock<TimerStateData>>,
    pub env: Arc<RwLock<EnvStateData>>,
    pub economy: Arc<RwLock<EconomyStateData>>,
    pub hub: HubState,
    pub misc: Arc<Mutex<MiscStateData>>,
    pub is_misc_updated: bool,
}

#[derive(Debug)]
pub struct StatePayload {
    pub timer: ReadOnlyRwLock<TimerStateData>,
    pub env: ReadOnlyRwLock<EnvStateData>,
    pub economy: ReadOnlyRwLock<EconomyStateData>,
    pub povver_plant: ReadOnlyRwLock<PovverPlantStateData>,
    pub factories: ReadOnlyRwLock<Vec<FactoryStateData>>,
    pub misc: Arc<Mutex<MiscStateData>>,
}

impl AppState {
    pub fn new(
        timer: Arc<RwLock<TimerStateData>>,
        env: Arc<RwLock<EnvStateData>>,
        economy: Arc<RwLock<EconomyStateData>>,
        hub: HubState,
        misc: Arc<Mutex<MiscStateData>>
    ) -> Self {

        Self {
            timer,
            env,
            economy,
            hub,
            misc,
            is_misc_updated: true,
        }
    }
}

impl AppState {
    pub fn get_state_payload(&self) -> Arc<StatePayload> {
        Arc::new(StatePayload {
            timer: ReadOnlyRwLock::from(Arc::clone(&self.timer)),
            env: ReadOnlyRwLock::from(Arc::clone(&self.env)),
            economy: ReadOnlyRwLock::from(Arc::clone(&self.economy)),
            povver_plant: ReadOnlyRwLock::from(Arc::clone(&self.hub.povver_plant)),
            factories: ReadOnlyRwLock::from(Arc::clone(&self.hub.factories)),
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