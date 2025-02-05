use std::{
    sync::{mpsc, Arc, Mutex}
};

use slint::ToSharedString;

use crate::{environment::Environment, speed::Speed, timer::Timer, ui_controller::UIController};
use crate::timer::TimerPayload;
use crate::ui_controller::{TimerData, UIState};

pub type SimInt = usize;
pub type SimFlo = f32;
pub type TickDuration = u64;

pub const DEFAULT_TICK_DURATION: TickDuration = 500;

pub enum UIFlag {
    Pause,
    Quit,
}

pub struct Simulation {
    timer: Timer,
    speed: Speed,
    env: Environment,
    ui_controller: UIController,
    entities: bool,
    is_running: bool,
    is_paused: bool,
}

impl Simulation {
    pub fn new() -> Self {
        let speed = Speed::NORMAL;
        let mut timer = Timer::new(speed.get_tick_duration(), 12);

        let timer_result = timer.tick(true);

        let ui_controller = UIController::new();

        Self {
            timer,
            speed,
            env: Environment::new(timer_result),
            ui_controller,
            entities: true,
            is_running: false,
            is_paused: true,
        }
    }
}

impl Simulation {
    fn get_ui_state(&self, timer_result: &TimerPayload) -> UIState {
        UIState {
            timer: TimerData {
                date: timer_result.date.into(),
                month_name: timer_result.month_data.name.to_shared_string(),
            },
            is_paused: self.is_paused,
        }
    }
}

impl Simulation {
    pub fn run(&mut self) {
        self.is_running = true;
        self.is_paused = false;

        let (ui_flag_sender, ui_flag_receiver) = mpsc::channel();
        let ui_state = Arc::new(Mutex::new(UIState::default()));
        let ui_join_handle = self
            .ui_controller
            .run(
                ui_flag_sender,
                Arc::clone(&ui_state)
            );

        loop {
            let flag_result = ui_flag_receiver.try_recv();
            if let Ok(flag) = flag_result {
                match flag {
                    UIFlag::Pause => self.toggle_paused(),
                    UIFlag::Quit => self.quit(),
                }
            }

            if !self.is_running {
                ui_join_handle.join().unwrap();
                break;
            }

            let timer_result = self.timer.tick(self.is_paused);
            if !self.is_paused {
                self.env.update(&timer_result);
            }

            let mut state_lock = ui_state.lock().unwrap();
            *state_lock = self.get_ui_state(&timer_result);
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
