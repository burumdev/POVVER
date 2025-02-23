use std::{
    sync::{Arc, mpsc},
    thread,
};

use crate::{
    app_state::StatePayload,
    economy::factory::Factory,
    economy::povver_plant::PovverPlant,
    simulation::StateAction,
};

pub struct TheHub {
    povver_plant: PovverPlant,
    factories: Vec<Factory>,
}

impl TheHub {
    pub fn new() -> Self {
        Self {
            povver_plant: PovverPlant::new(),
            factories: Vec::new(),
        }
    }
}

impl TheHub {
    pub fn start(
        &mut self,
        wakeup_receiver: mpsc::Receiver<StateAction>,
        state: Arc<StatePayload>,
    ) -> thread::JoinHandle<()> {
        self.povver_plant.start();

        thread::spawn(move || {
            loop {
                let action = wakeup_receiver.try_recv();

                if let Ok(action) = action {
                    match action {
                        StateAction::Timer => {
                            let timer_lock = state.timer.read().unwrap();
                        },
                        StateAction::Month => {},
                        StateAction::Env => {},
                        StateAction::Misc => {},
                        StateAction::Quit => {
                            println!("HUB: Quit signal received.");
                            break;
                        }
                    }
                }
            }
        })
    }
}