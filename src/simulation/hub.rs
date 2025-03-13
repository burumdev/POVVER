use std::{
    sync::{Arc, Mutex, RwLock},
    thread,
};
use std::time::Duration;
use crossbeam_channel::{Receiver, bounded};
use tokio::sync::broadcast as tokio_broadcast;

use crate::{
    app_state::{PovverPlantStateData, FactoryStateData, HubState, EconomyStateData, TimerStateData},
    economy::{
        povver_plant::PovverPlant,
    },
    simulation::{
        hub_types::*,
        hub_constants::*,
        hub_comms::*,
        StateAction,
        timer::TimerEvent,
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
    povver_plant: Arc<Mutex<PovverPlant>>,
    pub povver_plant_state: Arc<RwLock<PovverPlantStateData>>,
    pub factories_state: Arc<RwLock<Vec<FactoryStateData>>>,
    pub econ_state_ro: ReadOnlyRwLock<EconomyStateData>,
    pub timer_state_ro: ReadOnlyRwLock<TimerStateData>,
    pub hourly_jobs: Vec<HourlyJob>,
    pub daily_jobs: Vec<DailyJob>,
    ui_log_sender: tokio_broadcast::Sender<LogMessage>,
    comms: HubComms
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
                comms: HubComms::new(),
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
        let (pp_hub_signal_sender, pp_hub_signal_receiver) = bounded(1);

        let join_handles = {
            let mut me_lock = me.lock().unwrap();
            let handles = vec![
                PovverPlant::start(
                    Arc::clone(&me_lock.povver_plant),
                    me_lock.comms.broadcast_receiver(),
                    pp_hub_signal_sender,
                )
            ];
            me_lock.comms.broadcast_count = handles.len();

            handles
        };

        thread::spawn(move || {
            let mut sleeptime = Speed::NORMAL.get_tick_duration() / 2;
            loop {
                if let Ok(signal) = pp_hub_signal_receiver.try_recv() {
                    match signal {
                        PovverPlantSignal::BuyFuel(amount) => {
                            me.lock().unwrap().pp_buys_fuel(amount);
                        },
                        PovverPlantSignal::IncreaseFuelCapacity => {
                            me.lock().unwrap().pp_increases_fuel_capacity();
                        }
                    }
                }

                if let Ok(action) = wakeup_receiver.try_recv() {
                    me.lock().unwrap().comms.send_state_broadcast(action.clone());
                    match action {
                        StateAction::Timer(event) => {
                            let mut me_lock = me.lock().unwrap();
                            match event {
                                TimerEvent::DayChange => {
                                    me_lock.do_hourly_jobs();
                                    me_lock.do_daily_jobs();
                                }
                                TimerEvent::HourChange => {
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
                            let me_lock = me.lock().unwrap();
                            me_lock.log_console("Quit signal received.".to_string(), Warning);
                            me_lock.comms.send_state_broadcast(action);
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