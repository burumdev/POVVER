use std::{
    sync::{mpsc, Arc, RwLock},
    thread,
};
use tokio::sync::Notify;

use slint::{ModelRc, VecModel};
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
        state: Arc<RwLock<UIState>>,
        clouds: Arc<RwLock<Vec<Cloud>>>,
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
                let appw = app_weak.clone().unwrap();
                appw.window().set_maximized(true);

                loop {
                    state_notifier.notified().await;

                    appw.set_state(state.read().unwrap().clone());
                    let clouds_model_rc = ModelRc::from(clouds.read().unwrap().as_slice());
                    appw.set_clouds(clouds_model_rc);
                }
            }).unwrap();

            app.run().unwrap();
        })
    }
}
