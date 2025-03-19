use std::{
    thread,
    sync::{Arc, Mutex},
};
use std::time::Duration;
use crossbeam_channel::{Sender, Receiver};
use tokio::sync::broadcast as tokio_broadcast;

use crate::{
    app_state::{EconomyStateData, PovverPlantStateData},
    economy::economy_types::Money,
    utils_data::{SlidingWindow, ReadOnlyRwLock},
    simulation::{
        StateAction,
        timer::TimerEvent,
        hub_comms::{PPHubSignal, HubPPSignal, MessageEntity},
        SimInt,
        SimFlo,
        hub_constants::PP_FUEL_CAPACITY_INCREASE_COST,
        speed::Speed,
    },
    logger::{
        Logger,
        LogLevel::{Info, Warning, Critical},
        LogMessage,
    },
};

pub struct PovverPlant {
    fuel_buy_threshold: SimInt,
    last_ten_sales: SlidingWindow<Money>,
    state_ro: ReadOnlyRwLock<PovverPlantStateData>,
    econ_state_ro: ReadOnlyRwLock<EconomyStateData>,
    ui_log_sender: tokio_broadcast::Sender<LogMessage>,
    wakeup_receiver: Receiver<StateAction>,
    pp_hub_sender: Sender<PPHubSignal>,
    hub_pp_receiver: Receiver<HubPPSignal>,
}

impl PovverPlant {
    pub fn new(
        state_ro: ReadOnlyRwLock<PovverPlantStateData>,
        econ_state_ro: ReadOnlyRwLock<EconomyStateData>,
        ui_log_sender: tokio_broadcast::Sender<LogMessage>,
        wakeup_receiver: Receiver<StateAction>,
        pp_hub_sender: Sender<PPHubSignal>,
        hub_pp_receiver: Receiver<HubPPSignal>,
    ) -> Self {
        Self {
            fuel_buy_threshold: 5,
            last_ten_sales: SlidingWindow::new(10),
            state_ro,
            econ_state_ro,
            ui_log_sender,
            wakeup_receiver,
            pp_hub_sender,
            hub_pp_receiver,
        }
    }
}

impl PovverPlant {
    fn check_buy_fuel(&mut self) {
        let (is_awaiting_fuel, fuel) = {
            let state = self.state_ro.read().unwrap();
            (
                state.is_awaiting_fuel,
                state.fuel,
            )
        };
        match fuel {
            f if f <= self.fuel_buy_threshold => {
                if !is_awaiting_fuel {
                    self.log_ui_console("Fuel is low..".to_string(), Warning);
                    let (balance, fuel_capacity, fuel_price) = {
                        let state = self.state_ro.read().unwrap();
                        (
                            state.balance.val(),
                            state.fuel_capacity,
                            self.econ_state_ro.read().unwrap().fuel_price.val(),
                        )
                    };

                    let max_amount = balance / fuel_price;
                    if max_amount >= 1.0 {
                        let amount = (((max_amount / 10.0) + 1.0) as SimInt).clamp(0, fuel_capacity);
                        if amount == fuel_capacity {
                            self.maybe_upgrade_fuel_capacity(balance);
                        }
                        self.log_ui_console(format!("Buying fuel for amount {amount}"), Info);
                        self.pp_hub_sender.send(PPHubSignal::BuyFuel(amount)).unwrap();
                    }
                } else {
                    self.log_ui_console("Awaiting new fuel. Fuel level is critical!".to_string(), Critical);
                    println!();
                }
            },
            f if f > self.fuel_buy_threshold => {
                self.log_ui_console(format!("Fuel check completed. Amount {fuel} is sufficient."), Info);
            },
            _ => unreachable!()
        }
    }

    fn maybe_upgrade_fuel_capacity(&mut self, balance: SimFlo) {
        if (balance / 4.0) > PP_FUEL_CAPACITY_INCREASE_COST.val() {
            self.pp_hub_sender.send(PPHubSignal::IncreaseFuelCapacity).unwrap();
        }
    }
}

impl PovverPlant {
    pub fn start(
        me: Arc<Mutex<Self>>,
    ) -> thread::JoinHandle<()> {
        let (state_ro, wakeup_receiver, hub_pp_receiver) = {
            let me_lock = me.lock().unwrap();
            (
                ReadOnlyRwLock::clone(&me_lock.state_ro),
                me_lock.wakeup_receiver.clone(),
                me_lock.hub_pp_receiver.clone(),
            )
        };

        thread::spawn(move || {
            let mut sleeptime = Speed::NORMAL.get_tick_duration() / 2;
            loop {
                if let Ok(signal) = hub_pp_receiver.try_recv() {
                    match signal {
                        HubPPSignal::FuelTransfered => {
                            // Immediately resume production if we receive long awaited fuel!
                        }
                        HubPPSignal::FuelCapacityIncreased => {
                            // Fuel capacity increased. Let's do something about it!
                        }
                    }
                }
                if let Ok(action) = wakeup_receiver.try_recv() {
                    thread::sleep(Duration::from_micros(500));
                    if !state_ro.read().unwrap().is_bankrupt {
                        match action {
                            StateAction::Timer(TimerEvent::HourChange) => {
                                me.lock().unwrap().check_buy_fuel();
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
                    } else { // PP is BANKRUPT!
                        me.lock().unwrap().log_console("Gone belly up! We're bankrupt! Pivoting to potato salad production ASAP!".to_string(), Critical);
                        break;
                    }
                }
                thread::sleep(Duration::from_millis(sleeptime));
            }
        })
    }
}

impl Logger for PovverPlant {
    fn get_log_prefix(&self) -> String {
        "Povver Plant".to_string()
    }
    fn get_message_source(&self) -> MessageEntity {
        MessageEntity::PP
    }
    fn get_log_sender(&self) -> tokio_broadcast::Sender<LogMessage> {
        self.ui_log_sender.clone()
    }
}