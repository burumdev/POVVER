use std::{
    sync::{mpsc, Arc, Mutex}
};
use tokio::{
    sync::mpsc as tokio_mpsc,
};

use crate::{
    app_state::{AppState, TimerState, EnvState, MiscState},
    environment::Environment,
    economy::Economy,
    timer::{Timer, TimerEvent},
    ui_controller::{UIController, Date},
    speed::SPEEDS_ARRAY,
};

pub type SimInt = i32;
pub type SimFlo = f32;
pub type TickDuration = u64;

pub const DEFAULT_TICK_DURATION: TickDuration = 64;

#[derive(Debug)]
pub enum UIAction {
    Timer,
    Env,
    Misc(MiscState),
}

pub enum UIFlag {
    Pause,
    Quit,
    SpeedChange(SimInt),
}

#[derive(Debug)]
pub struct UIPayload {
    pub timer: Arc<Mutex<TimerState>>,
    pub env: Arc<Mutex<EnvState>>,
}

pub struct Simulation {
    app_state: AppState,
    timer: Timer,
    env: Environment,
    economy: Economy,
    ui_controller: UIController,
    speed_index: usize,
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
        let (mut timer, timer_state) = Timer::new(SPEEDS_ARRAY[speed_index].get_tick_duration(), init_date);
        timer.tick(is_paused);

        let (env, env_state) = Environment::new(Arc::clone(&timer_state));
        let economy = Economy::new();

        let misc_state = Arc::new(Mutex::new(MiscState {
            is_paused: true,
            speed_index,
        }));
        let app_state = AppState::new(timer_state, env_state, misc_state);
        let ui_controller = UIController::new();

        Self {
            app_state,
            timer,
            env,
            economy,
            ui_controller,
            speed_index,
            entities: true,
            is_running: false,
            is_paused,
        }
    }
}

impl Simulation {
    fn get_ui_state(&self) -> UIPayload {
        UIPayload {
            timer: Arc::clone(&self.app_state.timer),
            env: Arc::clone(&self.app_state.env),
        }
    }

    fn get_misc_state(&self) -> MiscState {
        MiscState {
            is_paused: self.is_paused,
            speed_index: self.speed_index,
        }
    }

    fn change_speed(&mut self, speed_index: SimInt) {
        self.speed_index = speed_index as usize;
        {
            let mut misc_lock = self.app_state.misc.lock().unwrap();
            misc_lock.speed_index = speed_index as usize;
        }
        self.timer.set_tick_duration(SPEEDS_ARRAY[self.speed_index].get_tick_duration());
    }
}

impl Simulation {
    pub fn run(&mut self) {
        self.is_running = true;
        self.is_paused = false;

        {
            let mut misc_lock = self.app_state.misc.lock().unwrap();
            misc_lock.is_paused = false;
        }

        let (ui_flag_sender, ui_flag_receiver) = mpsc::channel();
        let (ui_wakeup_sender, ui_wakeup_receiver) = tokio_mpsc::unbounded_channel();
        let ui_state = self.get_ui_state();

        let ui_join_handle = self
            .ui_controller
            .run(
                ui_flag_sender,
                ui_wakeup_receiver,
                ui_state,
            );

        ui_wakeup_sender.send(UIAction::Misc(self.get_misc_state())).unwrap();
        loop {
            let flag_result = ui_flag_receiver.try_recv();
            if let Ok(flag) = flag_result {
                match flag {
                    UIFlag::Pause => self.toggle_paused(),
                    UIFlag::Quit => self.quit(),
                    UIFlag::SpeedChange(speed_index) => self.change_speed(speed_index),
                }
                ui_wakeup_sender.send(UIAction::Misc(self.get_misc_state())).unwrap();
            }

            if !self.is_running {
                ui_join_handle.join().unwrap();
                break;
            }

            let timer_event = self.timer.tick(self.is_paused);
            ui_wakeup_sender.send(UIAction::Timer).unwrap();

            if !self.is_paused && timer_event != TimerEvent::NothingUnusual {
                self.env.update();
                println!("ENV updated: {:?}", self.env);
                ui_wakeup_sender.send(UIAction::Env).unwrap();
            }

            if timer_event == TimerEvent::MonthChange {
                self.economy.update_macroeconomics();
                println!("ECONOMY: {:?}", self.economy);
            }
        }
    }

    pub fn quit(&mut self) {
        self.is_running = false;
        println!("SIM: This simulation ended. Now yours continue.");
    }

    pub fn toggle_paused(&mut self) {
        let is_paused = !self.is_paused;
        self.is_paused = is_paused;
        {
            let mut misc_lock = self.app_state.misc.lock().unwrap();
            misc_lock.is_paused = is_paused;
        }

        if is_paused {
            println!("SIM: paused.");
        } else {
            println!("SIM: resuming stimulation.");
        }
    }
}
