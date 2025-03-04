use std::{
    thread,
    sync::{Arc, Mutex},
};
use crossbeam_channel::{Sender, Receiver};

use crate::{
    app_state::{EconomyStateData, PovverPlantStateData},
    economy::economy_types::Money,
    utils_data::{SlidingWindow, ReadOnlyRwLock},
    simulation::{
        StateAction,
        timer::TimerEvent,
        hub_types::PovverPlantSignal,
        SimInt,
        SimFlo,
        hub_constants::PP_FUEL_CAPACITY_INCREASE_COST
    },
};

pub struct PovverPlant {
    last_ten_sales: SlidingWindow<Money>,
    state_ro: ReadOnlyRwLock<PovverPlantStateData>,
    econ_state_ro: ReadOnlyRwLock<EconomyStateData>,
    fuel_buy_threshold: SimInt,
}

impl PovverPlant {
    pub fn new(
        state_ro: ReadOnlyRwLock<PovverPlantStateData>,
        econ_state_ro: ReadOnlyRwLock<EconomyStateData>
    ) -> Self {
        Self {
            last_ten_sales: SlidingWindow::new(10),
            state_ro,
            econ_state_ro,
            fuel_buy_threshold: 5,
        }
    }
}

impl PovverPlant {
    fn check_buy_fuel(&mut self, sender: &Sender<PovverPlantSignal>) {
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
                    println!("PP: Fuel is low");
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
                            self.maybe_upgrade_fuel_capacity(balance, sender);
                        }
                        println!("PP: Buying fuel for amount {amount}");
                        sender.send(PovverPlantSignal::BuyFuel(amount)).unwrap();
                    }
                } else {
                    println!("PP: Awaiting new fuel. Fuel level is critical!");
                }
            },
            f if f > self.fuel_buy_threshold => {
                println!("PP: Fuel check completed. Amount {fuel} is sufficient.");
            },
            _ => unreachable!()
        }
    }

    fn maybe_upgrade_fuel_capacity(&mut self, balance: SimFlo, sender: &Sender<PovverPlantSignal>) {
        if (balance / 4.0) > PP_FUEL_CAPACITY_INCREASE_COST.val() {
            sender.send(PovverPlantSignal::IncreaseFuelCapacity).unwrap();
        }
    }
}

impl PovverPlant {
    pub fn start(
        me: Arc<Mutex<Self>>,
        wakeup_receiver: Receiver<StateAction>,
        signal_sender: Sender<PovverPlantSignal>,
    ) -> thread::JoinHandle<()> {
        let state_ro = ReadOnlyRwLock::clone(&me.lock().unwrap().state_ro);

        thread::spawn(move || {
            while let Ok(action) = wakeup_receiver.recv() {
                if !state_ro.read().unwrap().is_bankrupt {
                    match action {
                        StateAction::Timer(TimerEvent::HourChange) => {
                            me.lock().unwrap()
                                .check_buy_fuel(&signal_sender);
                        },
                        StateAction::Quit => {
                            println!("PP: Quit signal received.");
                            break;
                        }
                        _ => ()
                    }
                } else { // PP is BANKRUPT!
                    println!("PP: Gone belly up! We're bankrupt! Pivoting to potato salad production ASAP!");
                    break;
                }
            }
        })
    }
}