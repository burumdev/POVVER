use std::{
    sync::{mpsc, Arc, RwLock},
    thread,
};
use crate::{
    app_state::{StatePayload, PovverPlantStateData},
    economy::{
        povver_plant::PovverPlant,
        economy_types::{EnergyUnit, Money}
    },
    simulation::StateAction,
    utils_data::ReadOnlyRwLock,
};
use crate::app_state::{FactoryStateData, HubState};

pub struct TheHub {
    povver_plant: PovverPlant,
    povver_plant_state: Arc<RwLock<PovverPlantStateData>>,
    factories_state: Arc<RwLock<Vec<FactoryStateData>>>,
}

impl TheHub {
    pub fn new() -> (Self, HubState) {
        let povver_plant_state = Arc::new(RwLock::new(PovverPlantStateData {
            fuel: 0,
            fuel_capacity: 200,
            production_capacity: EnergyUnit::new(400),
            balance: Money::new(10000.0),
        }));
        let factories_state = Arc::new(RwLock::new(Vec::new()));

        (
            Self {
                povver_plant: PovverPlant::new(ReadOnlyRwLock::from(Arc::clone(&povver_plant_state))),
                povver_plant_state: Arc::clone(&povver_plant_state),
                factories_state: Arc::clone(&factories_state),
            },
            HubState {
                povver_plant: povver_plant_state,
                factories: factories_state,
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
        let (pp_wakeup_sender, pp_wakeup_receiver) = mpsc::channel();
        self.povver_plant.start(pp_wakeup_receiver);

        thread::spawn(move || {
            loop {
                let action = wakeup_receiver.recv();

                if let Ok(action) = action {
                    pp_wakeup_sender.send(action.clone()).unwrap();
                    match action {
                        StateAction::Timer => {},
                        StateAction::Hour => {},
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
