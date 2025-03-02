use std::{
    sync::{Arc, Mutex, RwLock},
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
use crate::simulation::timer::TimerEvent;

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
    fn do_hourly_jobs(&mut self) {
        println!("Hey there doing hourly jobs!");
    }
}

impl TheHub {
    pub fn start(
        me: Arc<Mutex<Self>>,
        wakeup_receiver: crossbeam_channel::Receiver<StateAction>,
    ) -> thread::JoinHandle<()> {
        let mut broadcast_count = 0;

        let (broadcast_sender, broadcast_receiver) = crossbeam_channel::unbounded();
        let (pp_signal_sender, pp_signal_receiver) = crossbeam_channel::bounded(1);

        let (join_handles, econ_state, pp_state_mut) = {
            let mut me = me.lock().unwrap();
            (
                vec![
                    me.povver_plant.start(broadcast_receiver.clone(), pp_signal_sender)
                ],
                ReadOnlyRwLock::clone(&me.econ_state),
                Arc::clone(&me.povver_plant_state),
            )
        };

        broadcast_count += join_handles.len();

        let send_broadcast = move |action: StateAction| {
            for _ in 0..broadcast_count {
                if let Err(e) = broadcast_sender.send(action.clone()) {
                    eprintln!("HUB: Could not send action to one recipient: {e}");
                }
            }
        };

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

                if let Ok(action) = wakeup_receiver.recv() {
                    send_broadcast(action.clone());
                    match action {
                        StateAction::Timer(event) => {
                            match event {
                                TimerEvent::HourChange => {
                                    me.lock().unwrap().do_hourly_jobs();
                                }
                                _ => ()
                            }
                        },
                        StateAction::Env => {},
                        StateAction::Misc => {},
                        StateAction::Quit => {
                            println!("HUB: Quit signal received.");
                            send_broadcast(action);
                            for handle in join_handles {
                                handle.join().unwrap();
                            }
                            break;
                        }
                    }
                }
            }
        })
    }
}
