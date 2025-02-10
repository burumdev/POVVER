use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
};
use std::time::Duration;
use tokio::sync::Notify;

use slint::{ModelRc, VecModel, Timer, TimerMode};
use slint::CloseRequestResponse;

use crate::simulation::UIFlag;

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
        state: Arc<Mutex<UIState>>,
        clouds: Arc<Mutex<Vec<Cloud>>>,
        state_notifier: Arc<Notify>,
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
                let appw = app_weak.clone();
                let timer = Timer::default();
                loop {
                    state_notifier.notified().await;
                    timer.start(TimerMode::SingleShot, Duration::from_millis(3), || {});

                    let state_lock = state.lock().unwrap();
                    let clouds_lock = clouds.lock().unwrap();

                    let slice = (*clouds_lock).as_slice();
                    let clouds_model_rc = ModelRc::from(VecModel::from_slice(slice));

                    appw.unwrap().set_clouds(clouds_model_rc);
                    appw.unwrap().set_state((*state_lock).clone());
                }
            }).unwrap();

            // Start fullscreen
            let app_weak = app.as_weak();
            let fullscreen_handle = thread::spawn(move || {
                let appw = app_weak.clone();

                slint::invoke_from_event_loop(move || appw.unwrap().window().set_maximized(true))
                    .unwrap();
            });

            fullscreen_handle.join().unwrap();

            app.run().unwrap();
        })
    }
}
