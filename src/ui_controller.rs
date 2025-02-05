use slint::CloseRequestResponse;
use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
    time::Duration,
};
use slint::{ Timer, TimerMode };
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
        &mut self,
        flag_sender: mpsc::Sender<UIFlag>,
        state: Arc<Mutex<UIState>>,
    ) -> thread::JoinHandle<()> {
        let flag_sender_clone = flag_sender.clone();

        thread::spawn(move || {
            let app = PovverMain::new().unwrap();
            let timer = Timer::default();

            // Event handlers
            app.on_toggle_pause(move || {
                flag_sender.send(UIFlag::Pause).unwrap();
            });
            app.window().on_close_requested(move || {
                println!("UI: Shutting down the user interface");
                flag_sender_clone.send(UIFlag::Quit).unwrap();

                CloseRequestResponse::HideWindow
            });

            // Update state from simulation data
            let appw = app.as_weak().clone();
            timer.start(TimerMode::Repeated, Duration::from_millis(5), move || {
                let state_lock = state.lock().unwrap();

                appw.unwrap().set_state((*state_lock).clone());
            });

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
