use slint::CloseRequestResponse;
use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};
use slint::{ Timer, TimerMode };

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
        quit_flag: Arc<Mutex<bool>>,
        pause_flag: Arc<Mutex<bool>>,
        state: Arc<Mutex<UIState>>,
    ) -> thread::JoinHandle<()> {
        thread::spawn(move || {
            let app = PovverMain::new().unwrap();
            let timer = Timer::default();

            // Event handlers
            app.on_pause(move |paused| {
                let mut pause_lock = pause_flag.lock().unwrap();
                *pause_lock = paused;
            });

            // Update state from simulation data
            let app_weak = app.as_weak();
            timer.start(TimerMode::Repeated, Duration::from_millis(5), move || {
                let appw = app_weak.clone();

                let state_lock = state.lock().unwrap();

                appw.unwrap().set_state((*state_lock).clone());
            });

            // User clicked quit
            app.window().on_close_requested(move || {
                let mut quit = quit_flag.lock().unwrap();
                println!("UI: Shutting down the user interface");
                *quit = true;

                CloseRequestResponse::HideWindow
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
