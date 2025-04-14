use std::{
    sync::{Arc, Mutex, RwLock},
    thread,
};
use std::time::Duration;
use tokio::sync::broadcast as tokio_broadcast;

use crate::{
    app_state::{PovverPlantStateData, FactoryStateData, HubState, EconomyStateData, TimerStateData, EnvStateData},
    economy::{
        povver_plant::PovverPlant,
        factory::Factory,
        economy_types::{Money, EnergyUnit},
    },
    simulation::{
        SimFlo,
        hub_jobs::*,
        sim_constants::*,
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

use crate::simulation::test_factories::{get_test_factories, TEST_INDUSTRIES};

pub struct TheHub {
    pub povver_plant: Arc<Mutex<PovverPlant>>,
    pub povver_plant_state: Arc<RwLock<PovverPlantStateData>>,
    pub factories: Arc<Mutex<Vec<Arc<Mutex<Factory>>>>>,
    pub factories_state: Arc<RwLock<Vec<Arc<RwLock<FactoryStateData>>>>>,
    pub econ_state: Arc<RwLock<EconomyStateData>>,
    pub timer_state_ro: ReadOnlyRwLock<TimerStateData>,
    pub env_state_ro: ReadOnlyRwLock<EnvStateData>,
    pub minutely_jobs: Vec<MinutelyJob>,
    pub hourly_jobs: Vec<HourlyJob>,
    pub daily_jobs: Vec<DailyJob>,
    pub ui_log_sender: tokio_broadcast::Sender<LogMessage>,
    pub comms: HubComms
}

impl TheHub {
    pub fn new(
        econ_state: Arc<RwLock<EconomyStateData>>,
        timer_state_ro: ReadOnlyRwLock<TimerStateData>,
        env_state_ro: ReadOnlyRwLock<EnvStateData>,
        ui_log_sender: tokio_broadcast::Sender<LogMessage>,
    ) -> (Self, HubState) {
        // TODO: Test factories for now (from test-factories.rs). Maybe switch to user defined in the future.
        let comms = HubComms::new(TEST_INDUSTRIES.len());

        let povver_plant_state = Arc::new(RwLock::new(PovverPlantStateData {
            fuel: PP_INIT_FUEL_CAPACITY,
            fuel_capacity: PP_INIT_FUEL_CAPACITY,
            production_capacity: EnergyUnit::new(PP_INIT_PRODUCTION_CAP),
            balance: Money::new(PP_INIT_MONEY - (econ_state.read().unwrap().fuel_price.val() * PP_INIT_FUEL_CAPACITY as SimFlo)),
            is_awaiting_fuel: false,
            is_awaiting_fuel_capacity: false,
            is_awaiting_production_capacity: false,
            is_bankrupt: false,
        }));

        // TODO: Test factories for now (from test-factories.rs). Maybe switch to user defined in the future.
        let factories_state = get_test_factories();

        let (factories, to_factory_senders) = {
            let mut to_factory_senders = Vec::new();
            let factories = Arc::new(Mutex::new(
                factories_state
                    .iter()
                    .enumerate()
                    .map(|(id, f)| {
                        to_factory_senders.push(comms.clone_to_factory_dyn_sender(id));
                        Arc::new(Mutex::new(
                            Factory::new(
                                ReadOnlyRwLock::from(Arc::clone(f)),
                                ReadOnlyRwLock::from(Arc::clone(&econ_state)),
                                ReadOnlyRwLock::clone(&timer_state_ro),
                                ui_log_sender.clone(),
                                comms.clone_broadcast_state_receiver(),
                                comms.clone_broadcast_signal_receiver(),
                                comms.clone_from_factory_dyn_sender(id),
                                comms.clone_to_factory_dyn_receiver(id),
                            )
                        ))
                    }).collect()
            ));
            (factories, to_factory_senders)
        };

        let factories_state = Arc::new(RwLock::new(factories_state));

        let povver_plant = Arc::new(Mutex::new(PovverPlant::new(
            ReadOnlyRwLock::from(Arc::clone(&povver_plant_state)),
            ReadOnlyRwLock::from(Arc::clone(&econ_state)),
            ReadOnlyRwLock::clone(&timer_state_ro),
            ui_log_sender.clone(),
            comms.clone_broadcast_state_receiver(),
            comms.clone_pp_dyn_channel(),
            comms.clone_broadcast_signal_receiver(),
            to_factory_senders,
            comms.clone_from_factory_dyn_receivers()
        )));

        (
            Self {
                povver_plant,
                povver_plant_state: Arc::clone(&povver_plant_state),
                factories_state: Arc::clone(&factories_state),
                factories,
                econ_state,
                timer_state_ro,
                env_state_ro,
                minutely_jobs: Vec::new(),
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
    pub fn get_factory_state(&self, factory_id: usize) -> Option<Arc<RwLock<FactoryStateData>>> {
        let factories_state = self.factories_state.read().unwrap();
        let factory_state = factories_state.iter().find(|fac| fac.read().unwrap().id == factory_id);

        if let Some(factory) = factory_state {
            Some(factory.clone())
        } else {
            None
        }
    }
}

impl TheHub {
    pub fn start(
        me: Arc<Mutex<Self>>,
        mut wakeup_receiver: tokio_broadcast::Receiver<StateAction>,
    ) -> thread::JoinHandle<()> {
        let join_handles = {
            let me_lock = me.lock().unwrap();
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

            handles
        };

        let (pp_dyn_receiver, mut from_factory_dyn_receivers) = {
            let me_lock = me.lock().unwrap();
            (
                me_lock.comms.clone_pp_dyn_receiver(),
                me_lock.comms.clone_from_factory_dyn_receivers(),
            )
        };

        thread::Builder::new().name("POVVER_HUB".to_string()).spawn(move || {
            let mut sleeptime = ((Speed::NORMAL.get_tick_duration() / 2) * 1000) - 500;
            loop {
                if let Ok(signal) = pp_dyn_receiver.try_recv() {
                    let signal_any = signal.as_any();
                    match signal_any {
                        s if s.is::<PPHubSignal>() => {
                            let signal_from_pp = signal_any.downcast_ref::<PPHubSignal>().unwrap();
                            match signal_from_pp {
                                PPHubSignal::BuyFuel(amount) => {
                                    me.lock().unwrap().pp_buys_fuel(*amount);
                                },
                                PPHubSignal::ProduceEnergy(offer) => {
                                    me.lock().unwrap().pp_produces_energy(offer);
                                },
                                PPHubSignal::IncreaseFuelCapacity => {
                                    me.lock().unwrap().pp_increases_fuel_capacity();
                                },
                                PPHubSignal::IncreaseProductionCapacity => {
                                    me.lock().unwrap().pp_increases_production_capacity();
                                },
                            }
                        },
                        _ => ()
                    }
                }

                from_factory_dyn_receivers
                    .iter_mut()
                    .enumerate()
                    .for_each(|(fid, receiver)| {
                    if let Ok(signal) = receiver.try_recv() {
                        let signal_any = signal.as_any();
                        match signal_any {
                            s if s.is::<FactoryHubSignal>() => {
                                if let Some(signal_from_factory) = signal_any.downcast_ref::<FactoryHubSignal>() {
                                    match signal_from_factory {
                                        FactoryHubSignal::EnergyDemand(demand) => {
                                            me.lock().unwrap().factory_needs_energy(demand);
                                        },
                                        FactoryHubSignal::ProducingProductDemand(demand, units, unit_cost) => {
                                            me.lock().unwrap().factory_will_produce(fid, demand, *units, *unit_cost);
                                        },
                                        FactoryHubSignal::SellingProduct(stock_index, unit_price) => {
                                            me.lock().unwrap().factory_sells_product(fid, *stock_index, *unit_price);
                                        },
                                        FactoryHubSignal::BuyingSolarPanels(panels_count) => {
                                            me.lock().unwrap().factory_buys_solar_panels(fid, *panels_count);
                                        }
                                    }
                                }
                            },
                            _ => ()
                        }
                    }
                });

                if let Ok(action) = wakeup_receiver.try_recv() {
                    me.lock().unwrap().comms.send_state_broadcast(action.clone());

                    match action {
                        StateAction::Timer(event) => {
                            let mut me_lock = me.lock().unwrap();
                            if event.at_least_minute() {
                                me_lock.do_minutely_jobs();
                            }
                            if event.at_least_hour() {
                                me_lock.do_hourly_jobs(&event);
                            }
                            if event.at_least_day() {
                                me_lock.do_daily_jobs();
                            }
                        },
                        StateAction::SpeedChange(td) => {
                            sleeptime = ((td / 2) * 1000) - 500;
                        }
                        StateAction::Env => {},
                        StateAction::Misc => {},
                        StateAction::Quit => {
                            me.lock().unwrap().log_console("Quit signal received.".to_string(), Warning);
                            for handle in join_handles {
                                if let Err(e) = handle.join() {
                                    me.lock().unwrap().log_console(format!("Failed to join thread: {:?}", e), Warning);
                                }
                            }
                            break;
                        },
                        _ => (),
                    }
                }

                thread::sleep(Duration::from_micros(sleeptime));
            }
        }).unwrap()
    }
}

impl Logger for TheHub {
    fn get_log_prefix(&self) -> String {
        "HUB".to_string()
    }
    fn get_message_source(&self) -> MessageEntity {
        MessageEntity::Hub
    }
    fn get_log_sender(&self) -> &tokio_broadcast::Sender<LogMessage> {
        &self.ui_log_sender
    }
}