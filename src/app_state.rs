use std::{
    sync::{Arc, Mutex, RwLock},
};
use crate::{
    ui_controller::{Date, Cloud},
    months::MonthData
};
use crate::environment::{TheSun, WindSpeed};
use crate::ui_controller::WindDirection;

#[derive(Debug)]
pub struct TimerState {
    pub date: Date,
    pub month_data: &'static MonthData,
}

#[derive(Debug)]
pub struct EnvState {
    pub clouds: Vec<Cloud>,
    pub wind_speed: WindSpeed,
    pub wind_direction: WindDirection,
    pub the_sun: TheSun,
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
pub struct UIPayload {
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