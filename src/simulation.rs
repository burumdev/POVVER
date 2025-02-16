use std::{
    sync::{mpsc, Arc, RwLock},
};
use tokio::sync::Notify;
use slint::ToSharedString;

use crate::{
    environment::Environment,
    economy::Economy,
    timer::Timer,
    ui_controller::{UIController, TimerData, EnvData, UIState, Date, WindSpeedLevel},
    speed::SPEEDS_ARRAY,
};
use crate::timer::TimerEvent;

pub type SimInt = i32;
pub type SimFlo = f32;
pub type TickDuration = u64;

pub const DEFAULT_TICK_DURATION: TickDuration = 64;

pub enum UIFlag {
    Pause,
    Quit,
    SpeedChange(SimInt),
}

pub struct Simulation {
    timer: Timer,
    speed_index: usize,
    env: Environment,
    economy: Economy,
    ui_controller: UIController,
    entities: bool,
    is_running: bool,
    is_paused: bool,
}

impl Simulation {
    pub fn new() -> Self {
        let speed_index = 3;
        let init_date = Date {
            minute: 0,
            hour: 12,
            day: 1,
            month: 9,
            year: 2025,
        };
        let is_paused = true;
        let mut timer = Timer::new(SPEEDS_ARRAY[speed_index].get_tick_duration(), init_date);
        timer.tick(is_paused);

        let env = Environment::new(&timer);
        let economy = Economy::new();

        let ui_controller = UIController::new();

        Self {
            timer,
            speed_index,
            env,
            economy,
            ui_controller,
            entities: true,
            is_running: false,
            is_paused,
        }
    }
}

impl Simulation {
    fn get_ui_state(&self) -> UIState {
        UIState {
            timer: TimerData {
                date: self.timer.date.clone(),
            },
            env: EnvData {
                the_sun: self.env.the_sun.into(),
                wind_speed: self.env.wind_speed.val(),
                wind_direction: self.env.wind_direction,
                wind_speed_level: WindSpeedLevel::from(&self.env.wind_speed),
            },
            is_paused: self.is_paused,
            speed_index: self.speed_index as i32,
        }
    }

    fn change_speed(&mut self, speed_index: SimInt) {
        self.speed_index = speed_index as usize;
        self.timer.set_tick_duration(SPEEDS_ARRAY[self.speed_index].get_tick_duration());
    }
}

impl Simulation {
    pub fn run(&mut self) {
        self.is_running = true;
        self.is_paused = false;

        let (ui_flag_sender, ui_flag_receiver) = mpsc::channel();
        let ui_state = Arc::new(RwLock::new(self.get_ui_state()));
        let clouds = Arc::new(RwLock::new(self.env.clouds.clone()));
        let ui_state_notifier = Arc::new(Notify::new());

        let ui_join_handle = self
            .ui_controller
            .run(
                ui_flag_sender,
                Arc::clone(&ui_state),
                Arc::clone(&clouds),
                Arc::clone(&ui_state_notifier),
            );

        loop {
            let flag_result = ui_flag_receiver.try_recv();
            if let Ok(flag) = flag_result {
                match flag {
                    UIFlag::Pause => self.toggle_paused(),
                    UIFlag::Quit => self.quit(),
                    UIFlag::SpeedChange(speed_index) => self.change_speed(speed_index),
                }
            }

            if !self.is_running {
                ui_join_handle.join().unwrap();
                break;
            }

            let timer_event = self.timer.tick(self.is_paused);
            if !self.is_paused {
                self.env.update(&timer_event, &self.timer);
            }

            if timer_event == TimerEvent::HourChange {
                self.economy.update_macroeconomics();
                println!("ECONOMY: {:?}", self.economy);
            }

            let mut state_lock = ui_state.write().unwrap();
            *state_lock = self.get_ui_state();
            let mut clouds_lock = clouds.write().unwrap();
            *clouds_lock = self.env.clouds.clone();

            ui_state_notifier.notify_one();
        }
    }

    pub fn quit(&mut self) {
        self.is_running = false;
        println!("SIM: This simulation ended. Now yours continue.");
    }

    pub fn toggle_paused(&mut self) {
        self.is_paused = !self.is_paused;
        if self.is_paused {
            println!("SIM: paused.");
        } else {
            println!("SIM: resuming stimulation.");
        }
    }
}
