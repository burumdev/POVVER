use std::{
    sync::{Arc, Mutex, RwLock},
    thread,
};
use crossbeam_channel::{Receiver, bounded, unbounded};
use tokio::sync::broadcast as tokio_broadcast;

use crate::{
    app_state::{PovverPlantStateData, FactoryStateData, HubState, EconomyStateData, TimerStateData},
    economy::{
        povver_plant::PovverPlant,
        economy_types::{EnergyUnit, Money}
    },
    simulation::{
        hub_types::{PovverPlantSignal, HourlyJob, HourlyJobKind, DailyJob, DailyJobKind},
        hub_constants::*,
        StateAction,
        SimFlo,
        SimInt,
        timer::TimerEvent,
    },
    utils_data::ReadOnlyRwLock,
    logger::{
        Logger,
        LogLevel::{Info, Warning, Critical},
        LogMessage,
        MessageSource
    },
};

pub struct TheHub {
    povver_plant: Arc<Mutex<PovverPlant>>,
    povver_plant_state: Arc<RwLock<PovverPlantStateData>>,
    factories_state: Arc<RwLock<Vec<FactoryStateData>>>,
    econ_state_ro: ReadOnlyRwLock<EconomyStateData>,
    timer_state_ro: ReadOnlyRwLock<TimerStateData>,
    hourly_jobs: Vec<HourlyJob>,
    daily_jobs: Vec<DailyJob>,
    ui_log_sender: tokio_broadcast::Sender<LogMessage>,
}

impl TheHub {
    pub fn new(
        econ_state_ro: ReadOnlyRwLock<EconomyStateData>,
        timer_state_ro: ReadOnlyRwLock<TimerStateData>,
        ui_log_sender: tokio_broadcast::Sender<LogMessage>,
    ) -> (Self, HubState) {
        let povver_plant_state = Arc::new(RwLock::new(PovverPlantStateData {
            fuel: PP_INIT_FUEL,
            fuel_capacity: PP_INIT_FUEL_CAP,
            production_capacity: PP_INIT_PRODUCTION_CAP,
            balance: PP_INIT_MONEY,
            is_awaiting_fuel: false,
            is_bankrupt: false,
        }));
        let factories_state = Arc::new(RwLock::new(Vec::new()));

        let povver_plant = Arc::new(Mutex::new(PovverPlant::new(
            ReadOnlyRwLock::from(Arc::clone(&povver_plant_state)),
            ReadOnlyRwLock::clone(&econ_state_ro),
            ui_log_sender.clone(),
        )));

        (
            Self {
                povver_plant,
                povver_plant_state: Arc::clone(&povver_plant_state),
                factories_state: Arc::clone(&factories_state),
                econ_state_ro,
                timer_state_ro,
                hourly_jobs: Vec::new(),
                daily_jobs: Vec::new(),
                ui_log_sender,
            },
            HubState {
                povver_plant: povver_plant_state,
                factories: factories_state,
            },
        )
    }
}

impl TheHub {
    fn pp_buys_fuel(&mut self, amount: SimInt) {
        let price = self.econ_state_ro.read().unwrap().fuel_price;
        let fee = price.val() * amount as SimFlo;

        let transaction_successful =
            self.povver_plant_state.write().unwrap()
                .balance.dec(fee);

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
                self.log_ui_console(format!("PP bought fuel for amount {amount}"), Info);
                self.povver_plant_state.write().unwrap().is_awaiting_fuel = true;
            }
        } else {
            self.log_ui_console(format!("PP couldn't pay for fuel amount {amount} for the price of {fee}. Transaction canceled."), Warning);
        }
    }

    fn pp_increases_fuel_capacity(&mut self) {
        let transaction_successful =
            self.povver_plant_state.write().unwrap()
                .balance.dec(PP_FUEL_CAPACITY_INCREASE_COST.val());

        if transaction_successful {
            self.daily_jobs.push(DailyJob {
                kind: DailyJobKind::PPFuelCapIncrease,
                delay: 5,
                day_created: self.timer_state_ro.read().unwrap().date.day,
            });
            self.log_ui_console("PP is upgrading it's fuel capacity. ETA is 5 days.".to_string(), Info);
            println!();
        } else {
            self.log_ui_console("PP couldn't pay for fuel capacity increase. Upgrade canceled.".to_string(), Critical);
        }
    }
}

impl TheHub {
    fn do_hourly_jobs(&mut self) {
        self.log_console(format!("processing {} hourly jobs: {:?}", self.hourly_jobs.len(), self.hourly_jobs), Info);
        let this_hour = self.timer_state_ro.read().unwrap().date.hour;

        let mut due_jobs = Vec::new();
        self.hourly_jobs
            .retain_mut(|job| {
                if (job.hour_created + job.delay) % 23 == this_hour {
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

    fn do_daily_jobs(&mut self) {
        self.log_console(format!("processing {} daily jobs: {:?}", self.daily_jobs.len(), self.daily_jobs), Info);
        let today = self.timer_state_ro.read().unwrap().date.day;

        let mut due_jobs = Vec::new();
        self.daily_jobs
            .retain_mut(|job| {
                if (job.day_created + job.delay) % 30 == today {
                    due_jobs.push(job.clone());
                    return false;
                }

                true
            });

        for job in due_jobs {
            match job.kind {
                DailyJobKind::PPFuelCapIncrease => {
                    self.increase_pp_fuel_cap();
                }
            }
        }
    }

    fn transfer_fuel_to_pp(&self, amount: SimInt) {
        self.log_ui_console(format!("Transfering {amount} fuel to Povver Plant."), Info);

        let mut pp = self.povver_plant_state.write().unwrap();
        pp.fuel += amount;
        pp.is_awaiting_fuel = false;
    }

    fn increase_pp_fuel_cap(&self) {
        self.log_ui_console(format!("Increasing povver plant fuel capacity by {PP_FUEL_CAPACITY_INCREASE}."), Info);
        self.povver_plant_state.write().unwrap().fuel_capacity += PP_FUEL_CAPACITY_INCREASE;
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
                        },
                        PovverPlantSignal::IncreaseFuelCapacity => {
                            me.lock().unwrap().pp_increases_fuel_capacity();
                        }
                    }
                }

                if let Ok(action) = wakeup_receiver.recv() {
                    send_broadcast(action.clone());
                    match action {
                        StateAction::Timer(event) => {
                            match event {
                                TimerEvent::DayChange => {
                                    me.lock().unwrap().do_hourly_jobs();
                                    me.lock().unwrap().do_daily_jobs();
                                }
                                TimerEvent::HourChange => {
                                    me.lock().unwrap().do_hourly_jobs();
                                },
                                _ => ()
                            }
                        },
                        StateAction::Env => {},
                        StateAction::Misc => {},
                        StateAction::Quit => {
                            me.lock().unwrap().log_console("Quit signal received.".to_string(), Warning);
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

impl Logger for TheHub {
    fn get_log_prefix(&self) -> String {
        "HUB".to_string()
    }
    fn get_message_source(&self) -> MessageSource {
        MessageSource::Hub
    }
    fn get_log_sender(&self) -> tokio_broadcast::Sender<LogMessage> {
        self.ui_log_sender.clone()
    }
}