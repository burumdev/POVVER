use std::{
    sync::mpsc,
    thread,
};
use tokio::{
    sync::mpsc as tokio_mpsc,
};

use slint::{
    ModelRc,
    CloseRequestResponse,
};

use crate::{
    app_state::UIPayload,
    simulation::{SimInt, UIAction}
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
        mut wakeup_receiver: tokio_mpsc::UnboundedReceiver<UIAction>,
        state: UIPayload,
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

            let app_weak = app.as_weak();
            slint::spawn_local(async move {
                let appw = app_weak.clone().unwrap();
                appw.window().set_maximized(true);

                loop {
                    let action = wakeup_receiver.recv().await;

                    if let Some(action) = action {
                        match action {
                            UIAction::Timer => {
                                let timer_lock = state.timer.read().unwrap();
                                appw.set_timer(
                                    TimerData {
                                        date: timer_lock.date.clone(),
                                    }
                                )
                            },
                            UIAction::Env => {
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
                            UIAction::Misc(misc) => {
                                appw.set_state(UIState {
                                    is_paused: misc.is_paused,
                                    speed_index: misc.speed_index as SimInt,
                                });
                            }
                        }
                    }
                }
            }).unwrap();

            app.run().unwrap();
        })
    }
}
