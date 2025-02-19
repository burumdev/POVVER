use std::sync::{Arc, Mutex, RwLock};

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

#[derive(Debug)]
pub struct MiscState {
    pub is_paused: bool,
    pub speed_index: usize,
}

pub struct AppState {
    pub timer: Arc<RwLock<TimerState>>,
    pub env: Arc<RwLock<EnvState>>,
    pub misc: Arc<Mutex<MiscState>>,
}

impl AppState {
    pub fn new(
        timer: Arc<RwLock<TimerState>>,
        env: Arc<RwLock<EnvState>>,
        misc: Arc<Mutex<MiscState>>,
    ) -> Self {
        Self {
            timer,
            env,
            misc,
        }
    }
}