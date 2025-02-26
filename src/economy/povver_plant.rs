use std::{
    thread,
    sync::{mpsc, Arc, Mutex },
};
use crate::{
    app_state::PovverPlantStateData,
    economy::economy_types::Money,
    utils_data::SlidingWindow,
};
use crate::simulation::StateAction;
use crate::utils_data::ReadOnlyRwLock;

pub struct PovverPlant {
    last_ten_sales: Arc<Mutex<SlidingWindow<Money>>>,
    state: ReadOnlyRwLock<PovverPlantStateData>,
}

impl PovverPlant {
    pub fn new(state: ReadOnlyRwLock<PovverPlantStateData>) -> Self {
        Self {
            last_ten_sales: Arc::new(Mutex::new(SlidingWindow::new(10))),
            state
        }
    }
}

impl PovverPlant {
    pub fn start(&mut self, wakeup_receiver: mpsc::Receiver<StateAction>) -> thread::JoinHandle<()> {
        let state = ReadOnlyRwLock::clone(&self.state);
        let last_ten_sales = Arc::clone(&self.last_ten_sales);
        thread::spawn(move || {
            loop {
                let action = wakeup_receiver.recv();

                if let Ok(action) = action {
                    match action {
                        StateAction::Hour => {
                            if state.read().unwrap().fuel == 0 {
                                println!("Povver Plant: Fuel is low");
                            }
                        },
                        _ => ()
                    }
                }
            }
        })
    }
}