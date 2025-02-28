use std::{
    sync::{Arc, RwLock},
    thread,
};
use crate::{
    app_state::{PovverPlantStateData, FactoryStateData, HubState},
    economy::{
        povver_plant::PovverPlant,
        economy_types::{EnergyUnit, Money}
    },
    simulation::{
        hub_signals::PovverPlantSignals,
        StateAction
    },
    utils_data::ReadOnlyRwLock,
};
use crate::app_state::EconomyStateData;
use crate::simulation::SimFlo;

pub struct TheHub {
    povver_plant: PovverPlant,
    povver_plant_state: Arc<RwLock<PovverPlantStateData>>,
    factories_state: Arc<RwLock<Vec<FactoryStateData>>>,
}

impl TheHub {
    pub fn new() -> (Self, HubState) {
        let povver_plant_state = Arc::new(RwLock::new(PovverPlantStateData {
            fuel: 0,
            fuel_capacity: 50,
            production_capacity: EnergyUnit::new(400),
            balance: Money::new(10000.0),
            is_bankrupt: false,
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
        wakeup_receiver: crossbeam_channel::Receiver<StateAction>,
        econ_state: ReadOnlyRwLock<EconomyStateData>,
        pp_state_mut: Arc<RwLock<PovverPlantStateData>>,
    ) -> thread::JoinHandle<()> {
        let (pp_signal_sender, pp_signal_receiver) = crossbeam_channel::bounded(1);
        self.povver_plant.start(wakeup_receiver.clone(), ReadOnlyRwLock::clone(&econ_state), pp_signal_sender);

        thread::spawn(move || {
            loop {
                while let Ok(signal) = pp_signal_receiver.recv() {
                    match signal {
                        PovverPlantSignals::BuyFuel(amount) => {
                            println!("PP BUYS FUEL");
                            println!("Economy: {:?}", econ_state.read().unwrap());
                            let price = econ_state.read().unwrap().fuel_price;
                            let mut pp = pp_state_mut.write().unwrap();
                            if pp.balance.dec(amount as SimFlo * price.val()) {
                                pp.fuel += amount;
                            } else {
                                pp.is_bankrupt = true;
                            }
                            println!("Povver plant state: {:?}", pp);
                        }
                    }
                }
                while let Ok(action) = wakeup_receiver.recv() {
                    match action {
                        StateAction::Timer(_) => {},
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
