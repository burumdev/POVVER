use std::sync::{mpsc, Arc, Mutex};
use tokio::sync::mpsc as tokio_mpsc;

mod speed;
use speed::SPEEDS_ARRAY;
use hub::TheHub;

mod timer;
pub mod hub;
mod hub_signals;
mod hub_constants;

use timer::{Timer, TimerEvent};

use crate::{
    app_state::{AppState, Misc, MiscStateData},
    economy::Economy,
    environment::Environment,
    ui_controller::{Date, UIController, UIFlag},
};

pub type SimInt = i32;
pub type SimFlo = f32;
pub type TickDuration = u64;

pub const DEFAULT_TICK_DURATION: TickDuration = 64;

#[derive(Debug, Clone)]
pub enum StateAction {
    Timer,
    Month,
    Env,
    Misc,
    Quit
}

pub struct Simulation {
    app_state: AppState,
    timer: Timer,
    env: Environment,
    economy: Economy,
    ui_controller: UIController,
    the_hub: TheHub,
    is_running: bool,
}

impl Simulation {
    pub fn new() -> Self {
        let ui_controller = UIController::new();

        let speed_index = 3;
        let init_date = Date {
            minute: 0,
            hour: 12,
            day: 1,
            month: 1,
            year: 2025,
        };
        let is_paused = true;

        let (mut timer, timer_state) = Timer::new(SPEEDS_ARRAY[speed_index].get_tick_duration(), init_date);
        timer.tick(is_paused);
        let (env, env_state) = Environment::new(Arc::clone(&timer_state));
        let (economy, economy_state) = Economy::new();
        let (the_hub, hub_state) = TheHub::new();

        let misc_state = Arc::new(Mutex::new(MiscStateData {
            is_paused,
            speed_index,
        }));

        let app_state = AppState::new(timer_state, env_state, economy_state, hub_state, misc_state);


        Self {
            app_state,
            timer,
            env,
            economy,
            ui_controller,
            the_hub,
            is_running: false,
        }
    }
}

impl Simulation {
    fn change_speed(&mut self, speed_index: SimInt) {
        self.app_state.set_misc(Misc::SpeedIndex(speed_index as usize));
        self.timer.set_tick_duration(SPEEDS_ARRAY[speed_index as usize].get_tick_duration());
    }
}

impl Simulation {
    pub fn run(&mut self) {
        self.is_running = true;

        self.app_state.set_misc(Misc::IsPaused(false));

        let (ui_flag_sender, ui_flag_receiver) = mpsc::channel();
        let (wakeup_sender, wakeup_receiver) = mpsc::channel();
        let (ui_wakeup_sender, ui_wakeup_receiver) = tokio_mpsc::unbounded_channel();
        let state_payload = self.app_state.get_state_payload();

        let mut handles = Vec::new();
        handles.push(
            self.ui_controller
                .run(
                    ui_flag_sender,
                    ui_wakeup_receiver,
                    Arc::clone(&state_payload),
                )
        );

        handles.push(
            self.the_hub
                .start(wakeup_receiver, Arc::clone(&state_payload))
        );

        let send_action = |action: StateAction| {
            wakeup_sender.send(action.clone()).unwrap();
            ui_wakeup_sender.send(action).unwrap();
        };

        let mut misc = self.app_state.get_misc_state_updates().unwrap();
        send_action(StateAction::Misc);
        send_action(StateAction::Month);

        loop {
            if let Some(new_misc) = self.app_state.get_misc_state_updates() {
                misc = new_misc;
                send_action(StateAction::Misc);
            }

            let flag_result = ui_flag_receiver.try_recv();
            if let Ok(flag) = flag_result {
                match flag {
                    UIFlag::Pause => self.toggle_paused(),
                    UIFlag::Quit => self.quit(),
                    UIFlag::SpeedChange(speed_index) => self.change_speed(speed_index),
                }
            }

            if !self.is_running {
                send_action(StateAction::Quit);
                for handle in handles {
                    handle.join().unwrap();
                }
                break;
            }

            let timer_event = self.timer.tick(misc.is_paused);
            send_action(StateAction::Timer);
            if timer_event == TimerEvent::MonthChange {
                send_action(StateAction::Month);
            }

            if !misc.is_paused && timer_event != TimerEvent::NothingUnusual {
                self.env.update();
                println!("ENV updated: {:?}", self.env);
                send_action(StateAction::Env);
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
        let is_paused = !self.app_state.misc.lock().unwrap().is_paused;
        self.app_state.set_misc(Misc::IsPaused(is_paused));

        if is_paused {
            println!("SIM: paused.");
        } else {
            println!("SIM: resuming stimulation.");
        }
    }
}
