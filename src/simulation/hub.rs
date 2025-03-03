use std::{
    sync::{Arc, Mutex, RwLock},
    thread,
};
use crossbeam_channel::{Receiver, bounded, unbounded};
use crate::{
    app_state::{PovverPlantStateData, FactoryStateData, HubState},
    economy::{
        povver_plant::PovverPlant,
        economy_types::{EnergyUnit, Money}
    },
    simulation::{
        hub_types::PovverPlantSignal,
        StateAction
    },
    utils_data::ReadOnlyRwLock,
};
use crate::app_state::{EconomyStateData, TimerStateData};
use crate::simulation::hub_types::{HourlyJob, HourlyJobKind};
use crate::simulation::{SimFlo, SimInt};
use crate::simulation::timer::TimerEvent;

pub struct TheHub {
    povver_plant: Arc<Mutex<PovverPlant>>,
    povver_plant_state: Arc<RwLock<PovverPlantStateData>>,
    factories_state: Arc<RwLock<Vec<FactoryStateData>>>,
    econ_state_ro: ReadOnlyRwLock<EconomyStateData>,
    timer_state_ro: ReadOnlyRwLock<TimerStateData>,
    hourly_jobs: Vec<HourlyJob>,
}

impl TheHub {
    pub fn new(
        econ_state_ro: ReadOnlyRwLock<EconomyStateData>,
        timer_state_ro: ReadOnlyRwLock<TimerStateData>,
    ) -> (Self, HubState) {
        let povver_plant_state = Arc::new(RwLock::new(PovverPlantStateData {
            fuel: 0,
            fuel_capacity: 50,
            production_capacity: EnergyUnit::new(400),
            balance: Money::new(10000.0),
            is_awaiting_fuel: false,
            is_bankrupt: false,
        }));
        let factories_state = Arc::new(RwLock::new(Vec::new()));

        let povver_plant = Arc::new(Mutex::new(PovverPlant::new(
            ReadOnlyRwLock::from(Arc::clone(&povver_plant_state)),
            ReadOnlyRwLock::clone(&econ_state_ro),
        )));

        (
            Self {
                povver_plant,
                povver_plant_state: Arc::clone(&povver_plant_state),
                factories_state: Arc::clone(&factories_state),
                econ_state_ro,
                timer_state_ro,
                hourly_jobs: Vec::new()
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
        println!("HUB: processing {} hourly jobs: {:?}", self.hourly_jobs.len(), self.hourly_jobs);
        let this_hour = self.timer_state_ro.read().unwrap().date.hour;

        let mut due_jobs = Vec::new();
        self.hourly_jobs
            .retain_mut(|job| {
                if (job.hour_created + job.delay).clamp(0, 23) == this_hour {
                    due_jobs.push(job.clone());
                    return false;
                }

                true
            });

        for job in due_jobs {
            match job.kind {
                HourlyJobKind::PPBoughtFuel(amount) => {
                    self.transfer_fuel_to_pp(amount);
                }
            }
        }
    }

    fn pp_buys_fuel(&mut self, amount: SimInt) {
        println!("HUB: PP buys fuel for amount {amount}");
        let price = self.econ_state_ro.read().unwrap().fuel_price;
        let fee = price.val() * amount as SimFlo;

        let transaction_successful = {
            self.povver_plant_state.write().unwrap()
                .balance.dec(fee)
        };

        if transaction_successful {
            let delay = (amount as SimFlo / 5.0).floor() as SimInt;
            if delay == 0 {
                self.transfer_fuel_to_pp(amount);
            } else {
                let hour_created = self.timer_state_ro.read().unwrap().date.hour;
                self.hourly_jobs.push(
                    HourlyJob {
                        kind: HourlyJobKind::PPBoughtFuel(amount),
                        delay,
                        hour_created,
                    }
                );
                self.povver_plant_state.write().unwrap().is_awaiting_fuel = true;
            }
        } else {
            println!("HUB: PP couldn't pay for fuel amount {amount} for the price of {fee}. PP is BANKRUPT!");
            self.povver_plant_state.write().unwrap().is_bankrupt = true;
        }
    }

    fn transfer_fuel_to_pp(&self, amount: SimInt) {
        println!("HUB: Transfering {amount} fuel to PP.");

        let mut pp = self.povver_plant_state.write().unwrap();
        pp.fuel += amount;
        pp.is_awaiting_fuel = false;
    }
}

impl TheHub {
    pub fn start(
        me: Arc<Mutex<Self>>,
        wakeup_receiver: Receiver<StateAction>,
    ) -> thread::JoinHandle<()> {
        let mut broadcast_count = 0;

        let (broadcast_sender, broadcast_receiver) = unbounded();
        let (pp_signal_sender, pp_signal_receiver) = bounded(1);

        let join_handles = vec![
            PovverPlant::start(
                Arc::clone(&me.lock().unwrap().povver_plant),
                broadcast_receiver.clone(),
                pp_signal_sender
            )
        ];

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
                        PovverPlantSignal::BuyFuel(amount) => {
                            me.lock().unwrap().pp_buys_fuel(amount);
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
