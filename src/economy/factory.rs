use std::{
    thread,
    sync::{Arc, Mutex},
    time::Duration,
};
use crossbeam_channel::{Sender, Receiver};
use tokio::sync::broadcast as tokio_broadcast;

use crate::{
    app_state::{FactoryStateData, EconomyStateData},
    utils_data::ReadOnlyRwLock,
    simulation::{
        StateAction,
        hub_types::MessageEntity,
        SimInt,
        speed::Speed,
    },
    logger::{LogMessage, Logger, LogLevel::*},
};

pub struct Factory {
    state_ro: ReadOnlyRwLock<FactoryStateData>,
    econ_state_ro: ReadOnlyRwLock<EconomyStateData>,
    ui_log_sender: tokio_broadcast::Sender<LogMessage>,
    wakeup_receiver: Receiver<StateAction>,
}

impl Factory {
    pub fn new(
        state_ro: ReadOnlyRwLock<FactoryStateData>,
        econ_state_ro: ReadOnlyRwLock<EconomyStateData>,
        ui_log_sender: tokio_broadcast::Sender<LogMessage>,
        wakeup_receiver: Receiver<StateAction>
    ) -> Self {
        Self {
            state_ro,
            econ_state_ro,
            ui_log_sender,
            wakeup_receiver,
        }
    }
}

impl Factory {
    fn maybe_produce_goods(&self) {
        let producable_demands = {
            let econ_state_ro = self.econ_state_ro.read().unwrap();
            let state_ro = self.state_ro.read().unwrap();
            econ_state_ro
                .product_demands
                .iter()
                .copied()
                .filter(|demand| demand.product.industry == state_ro.industry)
                .filter(|demand| state_ro.product_portfolio.contains(&demand.product))
                .collect::<Vec<_>>()
        };

        if producable_demands.len() > 0 {
            self.log_console(format!("Producable demands: {:?}", producable_demands), Info);
        } else {
            self.log_console("No demands are producable".to_string(), Info);
        }
    }

    fn maybe_sell_goods(&self) {

    }
}

impl Factory {
    pub fn start(me: Arc<Mutex<Self>>) -> thread::JoinHandle<()> {
        let (state_ro, econ_state_ro, wakeup_receiver) = {
            let me_lock = me.lock().unwrap();
            (
                ReadOnlyRwLock::clone(&me_lock.state_ro),
                ReadOnlyRwLock::clone(&me_lock.econ_state_ro),
                me_lock.wakeup_receiver.clone(),
            )
        };

        thread::spawn(move || {
            let mut sleeptime = Speed::NORMAL.get_tick_duration() / 2;
            loop {
                if let Ok(action) = wakeup_receiver.try_recv() {
                    thread::sleep(Duration::from_micros(500));
                    if !state_ro.read().unwrap().is_bankrupt {
                        match action {
                            StateAction::Timer(event) => {
                                if event.at_least_hour() {
                                    me.lock().unwrap().log_console("Hour change from factory".to_string(), Info);
                                }
                                if event.at_least_minute() {
                                    me.lock().unwrap().maybe_produce_goods();
                                    me.lock().unwrap().maybe_sell_goods();
                                }
                            }
                            StateAction::SpeedChange(td) => {
                                sleeptime = td / 2;
                            }
                            StateAction::Quit => {
                                me.lock().unwrap().log_console("Quit signal received.".to_string(), Warning);
                                break;
                            }
                            _ => ()
                        }
                    } else { // Factory is BANKRUPT!
                        me.lock().unwrap().log_console("Gone belly up! We're bankrupt! Pivoting to ball bearing production ASAP!".to_string(), Critical);
                        break;
                    }
                }
                thread::sleep(Duration::from_millis(sleeptime));
            }
        })
    }
}

impl Logger for Factory {
    fn get_log_prefix(&self) -> String {
        format!("Factory No. {}", self.state_ro.read().unwrap().id)
    }
    fn get_message_source(&self) -> MessageEntity {
        MessageEntity::Factory(self.state_ro.read().unwrap().id as SimInt)
    }
    fn get_log_sender(&self) -> tokio_broadcast::Sender<LogMessage> {
        self.ui_log_sender.clone()
    }
}
