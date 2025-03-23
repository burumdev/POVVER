use std::{
    sync::{Arc, Mutex},
};

use tokio::sync::broadcast as tokio_broadcast;

pub mod speed;
use speed::SPEEDS_ARRAY;

pub mod hub;
use hub::TheHub;
pub mod hub_comms;
mod hub_events;
mod hub_jobs;
pub mod hub_constants;

pub mod timer;
mod sim_types;
pub use sim_types::*;

use timer::{Timer, TimerEvent};

use crate::{
    app_state::{AppState, Misc, MiscStateData},
    economy::Economy,
    environment::Environment,
    ui_controller::{Date, UIController, UIFlag},
    utils_data::ReadOnlyRwLock,
    logger::LogMessage,
};


#[derive(Debug, Clone)]
pub enum EconUpdate {
    Macro,
    Demands
}

#[derive(Debug, Clone)]
pub enum StateAction {
    Timer(TimerEvent),
    SpeedChange(TickDuration),
    EconUpdate(EconUpdate),
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
    the_hub: Arc<Mutex<TheHub>>,
    is_running: bool,
    ui_log_channel: (tokio_broadcast::Sender<LogMessage>, tokio_broadcast::Receiver<LogMessage>),
}

impl Simulation {
    pub fn new() -> Self {
        let ui_log_channel = tokio_broadcast::channel(128);
        let ui_controller = UIController::new();

        let speed_index = 3;
        let init_date = Date {
            minute: 0,
            hour: 12,
            day: 1,
            month: 7,
            year: 2025,
        };
        let is_paused = true;

        let (mut timer, timer_state) = Timer::new(SPEEDS_ARRAY[speed_index].get_tick_duration(), init_date);
        timer.tick(is_paused);

        let (mut env, env_state) = Environment::new(Arc::clone(&timer_state));
        env.update();

        let (economy, economy_state) = Economy::new(ReadOnlyRwLock::from(timer_state.clone()));

        let misc_state = Arc::new(Mutex::new(MiscStateData {
            is_paused,
            speed_index,
        }));

        let (the_hub, hub_state) = TheHub::new(
            ReadOnlyRwLock::from(economy_state.clone()),
            ReadOnlyRwLock::from(timer_state.clone()),
            ui_log_channel.0.clone()
        );
        let the_hub = Arc::new(Mutex::new(the_hub));


        let app_state = AppState::new(timer_state, env_state, economy_state, hub_state, misc_state);

        Self {
            app_state,
            timer,
            env,
            economy,
            ui_controller,
            the_hub,
            is_running: false,
            ui_log_channel,
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

        let (ui_flag_sender, ui_flag_receiver) = crossbeam_channel::bounded::<UIFlag>(16);
        let (wakeup_sender, wakeup_receiver) = tokio_broadcast::channel::<StateAction>(64);
        let log_receiver = self.ui_log_channel.1.resubscribe();
        let state_payload = self.app_state.get_state_payload();

        let join_handles = vec![
            self.ui_controller.run(
                ui_flag_sender,
                wakeup_receiver.resubscribe(),
                log_receiver,
                Arc::clone(&state_payload),
            ),
            TheHub::start(Arc::clone(&self.the_hub), wakeup_receiver),
        ];

        let broadcast_action = |action: StateAction| {
            if let Err(e) = wakeup_sender.send(action.clone()) {
                eprintln!("SIM: Could not deliver wakeup message to recipient: {e}");
            };
        };

        broadcast_action(StateAction::Timer(TimerEvent::MonthChange));
        broadcast_action(StateAction::Env);
        wakeup_sender.send(StateAction::Misc).unwrap();
        wakeup_sender.send(StateAction::EconUpdate(EconUpdate::Macro)).unwrap();
        self.economy.maybe_new_product_demands();
        broadcast_action(StateAction::EconUpdate(EconUpdate::Demands));

        let mut misc = self.app_state.get_misc_state_updates().unwrap();

        while self.timer.ticker.recv().is_ok() {
            if !self.is_running {
                // Send quit signal to every recipient for cleanups
                broadcast_action(StateAction::Quit);
                // Join all handles
                for handle in join_handles {
                    if let Err(e) = handle.join() {
                        eprintln!("SIM: Could not join thread: {:?}", e);
                    }
                }
                // Break out of main loop
                break;
            }

            let timer_event = self.timer.tick(misc.is_paused);
            match &timer_event {
                te if te.at_least_hour() => {
                    self.env.update();
                    broadcast_action(StateAction::Env);
                    self.economy.update_product_demands();

                    if te.at_least_day() {
                        self.economy.maybe_new_product_demands();
                    }
                    if te.at_least_month() {
                        self.economy.update_macroeconomics();
                        wakeup_sender.send(StateAction::EconUpdate(EconUpdate::Macro)).unwrap();
                    }

                    wakeup_sender.send(StateAction::EconUpdate(EconUpdate::Demands)).unwrap();
                },
                _ => ()
            }

            // Send timer signal to recipients to wake them up for timed jobs.
            broadcast_action(StateAction::Timer(timer_event));

            if let Some(new_misc) = self.app_state.get_misc_state_updates() {
                misc = new_misc;
                wakeup_sender.send(StateAction::Misc).unwrap();
            }

            if let Ok(flag) = ui_flag_receiver.try_recv() {
                match flag {
                    UIFlag::Pause => self.toggle_paused(),
                    UIFlag::SpeedChange(speed_index) => {
                        self.change_speed(speed_index);
                        wakeup_sender.send(StateAction::SpeedChange(self.timer.get_tick_duration())).unwrap();
                    },
                    UIFlag::Quit => self.quit(),
                }
            }
        }

        println!("SIM: This simulation ended. Now yours continue.");
    }

    pub fn quit(&mut self) {
        self.is_running = false;
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
