use crate::{environment::Environment, speed::Speed, timer::Timer, ui_controller::UIController};
use std::sync::{mpsc, Arc, Mutex};
use slint::ToSharedString;
use crate::ui_controller::{TimerData, UIState, Date as UIDate};

pub type SimInt = usize;
pub type SimFlo = f32;
pub type TickDuration = u64;

pub const DEFAULT_TICK_DURATION: TickDuration = 500;

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

        let timer_result = timer.tick();

        let ui_controller = UIController::new();

        Self {
            timer,
            speed,
            env: Environment::new(timer_result),
            ui_controller,
            entities: true,
            is_running: false,
            is_paused: false,
        }
    }
}

impl Simulation {
    pub fn run(&mut self) {
        self.is_running = true;

        let ui_quit_flag = Arc::new(Mutex::new(false));
        let ui_state = Arc::new(Mutex::new(UIState::default()));
        let ui_join_handle = self
            .ui_controller
            .run(Arc::clone(&ui_quit_flag), Arc::clone(&ui_state));

        loop {
            if !self.is_running {
                ui_join_handle.join().unwrap();
                break;
            }

            if !self.is_paused {
                let timer_result = self.timer.tick();
                self.env.update(&timer_result);

                let mut state_lock = ui_state.lock().unwrap();
                *state_lock = UIState {
                    timer: TimerData {
                        date: UIDate {
                            day: timer_result.date.day as i32,
                            hour: timer_result.date.hour as i32,
                            minute: timer_result.date.minute as i32,
                            month: timer_result.date.month as i32,
                            year: timer_result.date.year as i32,
                        },
                        month_name: timer_result.month_data.name.to_shared_string(),
                    }
                };
            }

            let quit = ui_quit_flag.lock().unwrap();
            if *quit {
                self.quit();
            }
        }
    }

    pub fn quit(&mut self) {
        self.is_running = false;
        println!("SIM: This simulation ended. Now yours continue.");
    }

    pub fn toggle_paused(&mut self) {
        self.is_paused = !self.is_paused;
    }
}
