use std::{
    sync::{Arc, Mutex, RwLock},
    thread,
};
use std::time::Duration;
use crossbeam_channel::Receiver;
use tokio::sync::broadcast as tokio_broadcast;

use crate::{
    app_state::{PovverPlantStateData, FactoryStateData, HubState, EconomyStateData, TimerStateData},
    economy::{
        povver_plant::PovverPlant,
        factory::Factory,
        industries::Industry,
        economy_types::Money,
        products::Product,
    },
    simulation::{
        hub_types::*,
        hub_constants::*,
        hub_comms::*,
        StateAction,
        speed::Speed,
    },
    utils_data::ReadOnlyRwLock,
    logger::{
        Logger,
        LogLevel::*,
        LogMessage,
    },
};

pub struct TheHub {
    pub povver_plant: Arc<Mutex<PovverPlant>>,
    pub povver_plant_state: Arc<RwLock<PovverPlantStateData>>,
    pub factories: Arc<Mutex<Vec<Arc<Mutex<Factory>>>>>,
    pub factories_state: Arc<RwLock<Vec<Arc<RwLock<FactoryStateData>>>>>,
    pub econ_state_ro: ReadOnlyRwLock<EconomyStateData>,
    pub timer_state_ro: ReadOnlyRwLock<TimerStateData>,
    pub hourly_jobs: Vec<HourlyJob>,
    pub daily_jobs: Vec<DailyJob>,
    pub ui_log_sender: tokio_broadcast::Sender<LogMessage>,
    pub comms: HubComms
}

impl TheHub {
    pub fn new(
        econ_state_ro: ReadOnlyRwLock<EconomyStateData>,
        timer_state_ro: ReadOnlyRwLock<TimerStateData>,
        ui_log_sender: tokio_broadcast::Sender<LogMessage>,
    ) -> (Self, HubState) {
        let comms = HubComms::new();

        let povver_plant_state = Arc::new(RwLock::new(PovverPlantStateData {
            fuel: PP_INIT_FUEL,
            fuel_capacity: PP_INIT_FUEL_CAP,
            production_capacity: PP_INIT_PRODUCTION_CAP,
            balance: PP_INIT_MONEY,
            is_awaiting_fuel: false,
            is_bankrupt: false,
        }));

        let industry_products = Product::by_industry(&Industry::SEMICONDUCTORS);
        let cheapest_rnd_product = industry_products
            .iter()
            .min_by(|prod_a, prod_b| prod_a.rnd_cost.val().total_cmp(&prod_b.rnd_cost.val())).unwrap();

        let product_portfolio = vec![*cheapest_rnd_product];

        let factories_state = vec![
            Arc::new(
                RwLock::new(
                    FactoryStateData {
                        balance: Money::new(FACTORY_INIT_MONEY.val() - product_portfolio[0].rnd_cost.val()),
                        industry: Industry::SEMICONDUCTORS,
                        product_portfolio,
                        id: 1,
                        is_bankrupt: false,
                    }
                )
            )
        ];

        let factories = Arc::new(Mutex::new(
            factories_state
                .iter()
                .map(|f|
                    Arc::new(Mutex::new(
                        Factory::new(
                            ReadOnlyRwLock::from(Arc::clone(f)),
                            ReadOnlyRwLock::clone(&econ_state_ro),
                            ui_log_sender.clone(),
                            comms.clone_broadcast_state_receiver(),
                            comms.clone_factory_hub_sender(),
                            comms.clone_broadcast_signal_receiver()
                        )
                    ))
                ).collect()
        ));

        let factories_state = Arc::new(RwLock::new(factories_state));

        let povver_plant = Arc::new(Mutex::new(PovverPlant::new(
            ReadOnlyRwLock::from(Arc::clone(&povver_plant_state)),
            ReadOnlyRwLock::clone(&econ_state_ro),
            ui_log_sender.clone(),
            comms.clone_broadcast_state_receiver(),
            comms.clone_pp_hub_sender(),
            comms.clone_hub_pp_receiver()
        )));

        (
            Self {
                povver_plant,
                povver_plant_state: Arc::clone(&povver_plant_state),
                factories_state: Arc::clone(&factories_state),
                factories,
                econ_state_ro,
                timer_state_ro,
                hourly_jobs: Vec::new(),
                daily_jobs: Vec::new(),
                ui_log_sender,
                comms
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
        me: Arc<Mutex<Self>>,
        wakeup_receiver: Receiver<StateAction>,
    ) -> thread::JoinHandle<()> {
        let join_handles = {
            let mut me_lock = me.lock().unwrap();
            let mut handles = vec![
                PovverPlant::start(
                    Arc::clone(&me_lock.povver_plant),
                )
            ];
            handles.extend(
                me_lock.factories.lock().unwrap()
                    .iter()
                    .map(|fac| Factory::start(Arc::clone(&fac)))
                    .collect::<Vec<thread::JoinHandle<()>>>()
                    .into_iter()
            );

            me_lock.comms.broadcast_count = handles.len();

            handles
        };

        let pp_hub_receiver = me.lock().unwrap().comms.clone_pp_hub_receiver();
        let factory_hub_receiver = me.lock().unwrap().comms.clone_factory_hub_receiver();
        thread::spawn(move || {
            let mut sleeptime = Speed::NORMAL.get_tick_duration() / 2;
            loop {
                if let Ok(signal) = pp_hub_receiver.try_recv() {
                    match signal {
                        PPHubSignal::BuyFuel(amount) => {
                            me.lock().unwrap().pp_buys_fuel(amount);
                        },
                        PPHubSignal::IncreaseFuelCapacity => {
                            me.lock().unwrap().pp_increases_fuel_capacity();
                        }
                    }
                }

                if let Ok(signal) = factory_hub_receiver.try_recv() {
                    match signal {
                        FactoryHubSignal::EnergyDemand(energy_demand) => {
                            me.lock().unwrap().factory_needs_energy(energy_demand);
                        },
                    }
                }

                if let Ok(action) = wakeup_receiver.try_recv() {
                    me.lock().unwrap().comms.send_state_broadcast(action.clone());
                    match action {
                        StateAction::Timer(event) => {
                            let mut me_lock = me.lock().unwrap();
                            match event {
                                e if e.at_least_day() => {
                                    me_lock.do_hourly_jobs();
                                    me_lock.do_daily_jobs();
                                }
                                e if e.at_least_hour() => {
                                    me_lock.do_hourly_jobs();
                                },
                                _ => ()
                            }
                        },
                        StateAction::SpeedChange(td) => {
                            sleeptime = td / 2;
                        }
                        StateAction::Env => {},
                        StateAction::Misc => {},
                        StateAction::Quit => {
                            me.lock().unwrap().log_console("Quit signal received.".to_string(), Warning);
                            for handle in join_handles {
                                handle.join().unwrap();
                            }
                            break;
                        },
                        _ => (),
                    }
                }

                thread::sleep(Duration::from_millis(sleeptime));
            }
        })
    }
}

impl Logger for TheHub {
    fn get_log_prefix(&self) -> String {
        "HUB".to_string()
    }
    fn get_message_source(&self) -> MessageEntity {
        MessageEntity::Hub
    }
    fn get_log_sender(&self) -> tokio_broadcast::Sender<LogMessage> {
        self.ui_log_sender.clone()
    }
}