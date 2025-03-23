use std::{
    thread,
    sync::{Arc, Mutex},
    time::Duration,
};
use crossbeam_channel::Sender;
use tokio::sync::broadcast as tokio_broadcast;

use crate::{
    app_state::{FactoryStateData, EconomyStateData},
    simulation::{
        StateAction,
        hub_comms::{MessageEntity, FactoryHubSignal, FactoryEnergyDemand, PPEnergyOffer, DynamicSignal},
        SimInt,
        speed::Speed,
    },
    economy::economy_types::EnergyUnit,
    logger::{LogMessage, Logger, LogLevel::*},
    utils_traits::AsFactor,
    utils_data::ReadOnlyRwLock,
};

pub struct Factory {
    state_ro: ReadOnlyRwLock<FactoryStateData>,
    econ_state_ro: ReadOnlyRwLock<EconomyStateData>,
    ui_log_sender: tokio_broadcast::Sender<LogMessage>,
    wakeup_receiver: tokio_broadcast::Receiver<StateAction>,
    factory_hub_sender: Sender<FactoryHubSignal>,
    hub_broadcast_receiver: tokio_broadcast::Receiver<DynamicSignal>,
    hub_signal_receiver: crossbeam_channel::Receiver<DynamicSignal>,
}

impl Factory {
    pub fn new(
        state_ro: ReadOnlyRwLock<FactoryStateData>,
        econ_state_ro: ReadOnlyRwLock<EconomyStateData>,
        ui_log_sender: tokio_broadcast::Sender<LogMessage>,
        wakeup_receiver: tokio_broadcast::Receiver<StateAction>,
        factory_hub_sender: Sender<FactoryHubSignal>,
        hub_broadcast_receiver: tokio_broadcast::Receiver<DynamicSignal>,
        hub_signal_receiver: crossbeam_channel::Receiver<DynamicSignal>,
    ) -> Self {
        Self {
            state_ro,
            econ_state_ro,
            ui_log_sender,
            wakeup_receiver,
            factory_hub_sender,
            hub_broadcast_receiver,
            hub_signal_receiver
        }
    }
}

impl Factory {
    fn maybe_produce_goods(&self) {
        let (factory_id, producable_demands) = {
            let econ_state_ro = self.econ_state_ro.read().unwrap();
            let state_ro = self.state_ro.read().unwrap();
            (
                state_ro.id,
                econ_state_ro
                    .product_demands
                    .iter()
                    .copied()
                    .filter(|demand| demand.product.industry == state_ro.industry && state_ro.product_portfolio.contains(&demand.product))
                    .collect::<Vec<_>>()
            )
        };

        if producable_demands.len() > 0 {
            for demand in producable_demands {
                let product = &demand.product;
                let production_cost = &product.unit_production_cost;
                let demand_info = &product.demand_info;

                let units = demand_info.unit_per_percent * (demand.percent.val() as SimInt);
                let energy_needed = product.unit_production_cost.energy.val() * units;

                self.factory_hub_sender.send(
                    FactoryHubSignal::EnergyDemand(
                        FactoryEnergyDemand {
                            factory_id,
                            energy: EnergyUnit::new(energy_needed),
                        }
                    )
                ).unwrap();
            }
        } else {
            self.log_console("No demands are producable".to_string(), Info);
        }
    }

    fn maybe_sell_goods(&self) {

    }
}

impl Factory {
    pub fn start(me: Arc<Mutex<Self>>) -> thread::JoinHandle<()> {
        let (my_id, state_ro, econ_state_ro, mut wakeup_receiver, mut hub_broadcast_receiver, hub_signal_receiver) = {
            let me_lock = me.lock().unwrap();
            (
                me_lock.state_ro.read().unwrap().id,
                ReadOnlyRwLock::clone(&me_lock.state_ro),
                ReadOnlyRwLock::clone(&me_lock.econ_state_ro),
                me_lock.wakeup_receiver.resubscribe(),
                me_lock.hub_broadcast_receiver.resubscribe(),
                me_lock.hub_signal_receiver.clone(),
            )
        };

        thread::spawn(move || {
            let mut sleeptime = Speed::NORMAL.get_tick_duration() / 2;
            loop {
                if let Ok(signal) = hub_broadcast_receiver.try_recv() {
                    let signal_any = signal.as_any();
                    match signal_any {
                        s if s.is::<FactoryEnergyDemand>() => {
                            if let Some(demand) = signal_any.downcast_ref::<FactoryEnergyDemand>() {
                                if demand.factory_id != my_id {
                                    //TODO
                                    //me.lock().unwrap().log_console(format!("Got message: {:?} is from another guy :)", signal), Critical);
                                    // MAYBE SELL SOME LEFTOVER ENERGY TO THE FACTORY IN NEED
                                } else {
                                    //TODO
                                    //me.lock().unwrap().log_console(format!("Got message: {:?} is from me haha :)", signal), Critical);
                                }
                            } else {
                                me.lock().unwrap().log_console("Could not downcast broadcast signal from hub!".to_string(), Error);
                            }
                        },
                        _ => {
                            //TODO
                            me.lock().unwrap().log_console(format!("Got message: {:?}. But is not an energy demand?", signal), Critical);
                        }
                    }
                }

                if let Ok(signal) = hub_signal_receiver.try_recv() {
                    let signal_any = signal.as_any();
                    match signal_any {
                        s if s.is::<PPEnergyOffer>() => {
                            if let Some(offer) = signal_any.downcast_ref::<PPEnergyOffer>() {
                                me.lock().unwrap().log_console(format!("Got energy offer from PP: {:?}.", offer), Info);
                            } else {
                                me.lock().unwrap().log_console("Could not downcast broadcast signal from hub!".to_string(), Error);
                            }
                        },
                        _ => {
                            //TODO
                            me.lock().unwrap().log_console(format!("Got message: {:?}. But is not an energy demand?", signal), Critical);
                        }
                    }
                }

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
