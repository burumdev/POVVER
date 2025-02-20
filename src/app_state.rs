use std::{
    sync::{Arc, RwLock},
    collections::HashMap,
    any::Any
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
pub struct MiscState {
    pub is_paused: bool,
    pub speed_index: usize,
}
pub type MiscMap = HashMap<&'static str, Box<dyn Any>>;

impl From<&MiscMap> for MiscState {
    fn from(map: &MiscMap) -> MiscState {
        MiscState {
            is_paused: map.get("is_paused").unwrap().downcast_ref::<bool>().unwrap().clone(),
            speed_index: map.get("speed_index").unwrap().downcast_ref::<usize>().unwrap().clone(),
        }
    }
}
pub struct AppState {
    pub timer: Arc<RwLock<TimerState>>,
    pub env: Arc<RwLock<EnvState>>,
    pub misc: MiscMap,
    pub misc_changed: bool,
}

#[derive(Debug)]
pub struct UIPayload {
    pub timer: Arc<RwLock<TimerState>>,
    pub env: Arc<RwLock<EnvState>>,
}

impl AppState {
    pub fn new(
        timer: Arc<RwLock<TimerState>>,
        env: Arc<RwLock<EnvState>>,
    ) -> Self {
        let mut misc_map = MiscMap::new();
        misc_map.insert("is_paused", Box::new(true));
        misc_map.insert("speed_index", Box::new(3usize));
        let misc = misc_map;

        Self {
            timer,
            env,
            misc,
            misc_changed: true,
        }
    }
}

impl AppState {
    pub fn get_misc_state_updates(&mut self) -> Option<MiscState> {
        if self.misc_changed {
            self.misc_changed = false;

            Some(MiscState::from(&self.misc))
        } else {
            None
        }
    }

    pub fn get_misc<T: Any + Clone>(&self, key: &str) -> Result<T, String> {
        let value = {
            let found = self.misc
                .get(key)
                .ok_or(format!("Misc state: value with key '{}' not found", key))?;

            found.downcast_ref::<T>()
                .ok_or("Misc state: type mismatch in get_misc.")?
                .clone()
        };

        Ok(value)
    }

    pub fn set_misc<T: Any + Clone>(&mut self, key: &str, val: T) -> Result<(), String> {
        let found = self.misc
            .get_mut(key)
            .ok_or(format!("Misc state: value with key '{}' not found", key))?;

        *found.downcast_mut::<T>()
            .ok_or("Misc state: type mismatch in set_misc")? = val;

        self.misc_changed = true;

        Ok(())
    }
}