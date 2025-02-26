use std::{
    sync::{mpsc, Arc, RwLock},
    thread,
};
use crate::{
    app_state::{StatePayload, PovverPlantStateData},
    economy::{
        factory::Factory,
        povver_plant::PovverPlant,
        economy_types::{EnergyUnit, Money}
    },
    simulation::StateAction,
    utils_data::ReadOnlyRwLock,
};
use crate::app_state::HubState;

pub struct TheHub {
    povver_plant: PovverPlant,
    povver_plant_state: Arc<RwLock<PovverPlantStateData>>,
    factories: Vec<Factory>,
}

impl TheHub {
    pub fn new() -> (Self, HubState) {
        let povver_plant_state = Arc::new(RwLock::new(PovverPlantStateData {
            fuel: 0,
            fuel_capacity: 200,
            production_capacity: EnergyUnit::new(400),
            balance: Money::new(10000.0),
        }));
        
        (
            Self {
                povver_plant: PovverPlant::new(ReadOnlyRwLock::from(Arc::clone(&povver_plant_state))),
                povver_plant_state: Arc::clone(&povver_plant_state),
                factories: Vec::new(),
            },
            HubState {
                povver_plant: povver_plant_state,
                factories: Arc::new(Vec::new()),
            },
        )
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
                        StateAction::Timer => {},
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
