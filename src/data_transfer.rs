use std::sync::Arc;
use crate::{
    simulation::SimInt,
    ui_controller::{Date, WindDirection, WindSpeedLevel},
    environment::{TheSun, WindSpeed},
};

pub struct UIPayload {
    pub date: Arc<Date>,
    pub the_sun: Arc<TheSun>,
    pub wind_speed: Arc<WindSpeed>,
    pub wind_direction: Arc<WindDirection>,
    pub wind_speed_level: Arc<WindSpeedLevel>,
    pub is_paused: Arc<bool>,
    pub speed_index: Arc<SimInt>,
}