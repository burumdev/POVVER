use std::{
    sync::{Arc, mpsc},
    thread,
};
use std::sync::RwLock;
use crate::{
    app_state::{StatePayload, PovverPlantStateData},
    economy::{
        factory::Factory,
        povver_plant::PovverPlant,
        economy_types::{EnergyUnit, Money}
    },
    simulation::StateAction,
};

pub struct TheHub {
    povver_plant: PovverPlant,
    povver_plant_state: Arc<RwLock<PovverPlantStateData>>,
    factories: Vec<Factory>,
}

impl TheHub {
    pub fn new() -> (Self, Arc<RwLock<PovverPlantStateData>>) {
        let povver_plant_state = Arc::new(RwLock::new(PovverPlantStateData {
            fuel: 0,
            fuel_capacity: 200,
            production_capacity: EnergyUnit::new(400),
            balance: Money::new(10000.0),
        }));

        (
            Self {
                povver_plant: PovverPlant::new(),
                povver_plant_state: Arc::clone(&povver_plant_state),
                factories: Vec::new(),
            },
            povver_plant_state,
        )
    }
}

impl TheHub {
    pub fn start(
        &mut self,
        wakeup_receiver: mpsc::Receiver<StateAction>,
        state: Arc<StatePayload>,
    ) -> thread::JoinHandle<()> {

        self.povver_plant.start(Arc::clone(&self.povver_plant_state));

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