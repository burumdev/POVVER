use std::{
    sync::{mpsc, Arc},
    thread,
};

use tokio::{
    sync::mpsc as tokio_mpsc,
};

use slint::{ModelRc, CloseRequestResponse, SharedString};

use crate::{
    app_state::StatePayload,
    simulation::{SimInt, StateAction}
};

pub enum UIFlag {
    Pause,
    Quit,
    SpeedChange(SimInt),
}

slint::include_modules!();

pub struct UIController {}

impl UIController {
    pub fn new() -> Self {
        Self {}
    }
}

impl UIController {
    pub fn run(
        &self,
        flag_sender: mpsc::Sender<UIFlag>,
        mut wakeup_receiver: tokio_mpsc::UnboundedReceiver<StateAction>,
        state: Arc<StatePayload>,
    ) -> thread::JoinHandle<()> {
        let flag_sender_close = flag_sender.clone();
        let flag_sender_speed = flag_sender.clone();

        thread::spawn(move || {
            let app = PovverMain::new().unwrap();

            // Event handlers
            app.on_toggle_pause(move || {
                flag_sender.send(UIFlag::Pause).unwrap();
            });
            app.on_speed_change(move |speed_index| {
                flag_sender_speed.send(UIFlag::SpeedChange(speed_index)).unwrap();
            });
            app.window().on_close_requested(move || {
                println!("UI: Shutting down the user interface");
                flag_sender_close.send(UIFlag::Quit).unwrap();

                CloseRequestResponse::HideWindow
            });

            // UI state updates
            let app_weak = app.as_weak();
            slint::spawn_local(async move {
                let appw = app_weak.clone().unwrap();
                appw.window().set_maximized(true);

                loop {
                    let action = wakeup_receiver.recv().await;

                    if let Some(action) = action {
                        match action {
                            StateAction::Timer => {
                                let timer_lock = state.timer.read().unwrap();
                                appw.set_timer(
                                    TimerData {
                                        date: timer_lock.date.clone(),
                                    }
                                );
                            },
                            StateAction::Month => {
                                let timer_lock = state.timer.read().unwrap();
                                appw.set_month(
                                    MonthData {
                                        day_start: timer_lock.month_data.day_start,
                                        day_end: timer_lock.month_data.day_end,
                                        name: SharedString::from(timer_lock.month_data.name),
                                        sunshine_factor: timer_lock.month_data.sunshine_factor,
                                        windspeed_factor: timer_lock.month_data.cloud_forming_factor,
                                        cloud_forming_factor: timer_lock.month_data.cloud_forming_factor,
                                    }
                                );
                            },
                            StateAction::Env => {
                                let env_lock = state.env.read().unwrap();
                                appw.set_env(
                                    EnvData {
                                        the_sun: env_lock.the_sun.into(),
                                        wind_direction: env_lock.wind_direction,
                                        wind_speed: env_lock.wind_speed.val(),
                                        wind_speed_level: WindSpeedLevel::from(&env_lock.wind_speed),
                                        clouds: ModelRc::from(env_lock.clouds.as_slice()),
                                    }
                                );
                            },
                            StateAction::Misc => {
                                let misc_lock = state.misc.lock().unwrap();
                                appw.set_state(UIState {
                                    is_paused: misc_lock.is_paused,
                                    speed_index: misc_lock.speed_index as SimInt,
                                });
                            },
                            StateAction::Quit => break,
                            _ => ()
                        }
                    }
                }
            }).unwrap();

            // Run the UI
            app.run().unwrap();
        })
    }
}
