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
    econ_state: ReadOnlyRwLock<EconomyStateData>,
}

impl TheHub {
    pub fn new(econ_state: ReadOnlyRwLock<EconomyStateData>) -> (Self, HubState) {
        let povver_plant_state = Arc::new(RwLock::new(PovverPlantStateData {
            fuel: 0,
            fuel_capacity: 50,
            production_capacity: EnergyUnit::new(400),
            balance: Money::new(10000.0),
            is_bankrupt: false,
        }));
        let factories_state = Arc::new(RwLock::new(Vec::new()));

        let povver_plant = PovverPlant::new(
            ReadOnlyRwLock::from(Arc::clone(&povver_plant_state)),
            ReadOnlyRwLock::clone(&econ_state),
        );

        (
            Self {
                povver_plant,
                povver_plant_state: Arc::clone(&povver_plant_state),
                factories_state: Arc::clone(&factories_state),
                econ_state,
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
    ) -> thread::JoinHandle<()> {
        let (pp_signal_sender, pp_signal_receiver) = crossbeam_channel::bounded(1);

        let join_handles = vec![
            self.povver_plant.start(wakeup_receiver.clone(), pp_signal_sender)
        ];

        let econ_state = ReadOnlyRwLock::clone(&self.econ_state);
        let pp_state_mut = Arc::clone(&self.povver_plant_state);
        thread::spawn(move || {
            loop {
                if let Ok(signal) = pp_signal_receiver.try_recv() {
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

                break;
            }

            for handle in join_handles {
                handle.join().unwrap();
            }
        })
    }
}
