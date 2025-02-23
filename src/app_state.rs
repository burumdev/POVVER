use std::sync::{Arc, Mutex, RwLock};
use crate::{
    environment::{TheSun, WindSpeed},
    simulation::SimFlo,
    ui_controller::{Cloud, Date, WindDirection},
};
use crate::environment::months::Month;

#[derive(Debug)]
pub struct TimerState {
    pub date: Date,
    pub month_data: &'static Month,
}

#[derive(Debug)]
pub struct EnvState {
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
pub struct MiscState {
    pub is_paused: bool,
    pub speed_index: usize,
}

pub struct AppState {
    pub timer: Arc<RwLock<TimerState>>,
    pub env: Arc<RwLock<EnvState>>,
    pub misc: Arc<Mutex<MiscState>>,
    pub is_misc_updated: bool,
}

#[derive(Debug)]
pub struct StatePayload {
    pub timer: Arc<RwLock<TimerState>>,
    pub env: Arc<RwLock<EnvState>>,
    pub misc: Arc<Mutex<MiscState>>,
}

impl AppState {
    pub fn new(
        timer: Arc<RwLock<TimerState>>,
        env: Arc<RwLock<EnvState>>,
        misc: Arc<Mutex<MiscState>>
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

    pub fn get_misc_state_updates(&mut self) -> Option<MiscState> {
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