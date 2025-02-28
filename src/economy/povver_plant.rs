use std::{
    thread,
    sync::{Arc, Mutex},
};
use crate::{
    app_state::PovverPlantStateData,
    economy::economy_types::Money,
    utils_data::SlidingWindow,
    simulation::{
        StateAction,
        timer::TimerEvent,
        hub_signals::PovverPlantSignals,
    },
    utils_data::ReadOnlyRwLock,
};
use crate::app_state::EconomyStateData;
use crate::simulation::SimInt;

pub struct PovverPlant {
    last_ten_sales: Arc<Mutex<SlidingWindow<Money>>>,
    state: ReadOnlyRwLock<PovverPlantStateData>,
}

impl PovverPlant {
    pub fn new(state: ReadOnlyRwLock<PovverPlantStateData>) -> Self {
        Self {
            last_ten_sales: Arc::new(Mutex::new(SlidingWindow::new(10))),
            state
        }
    }
}

impl PovverPlant {
    pub fn start(
        &mut self,
        wakeup_receiver: crossbeam_channel::Receiver<StateAction>,
        econ_state: ReadOnlyRwLock<EconomyStateData>,
        signal_sender: crossbeam_channel::Sender<PovverPlantSignals>,
    ) -> thread::JoinHandle<()> {
        let state = ReadOnlyRwLock::clone(&self.state);
        let last_ten_sales = Arc::clone(&self.last_ten_sales);
        thread::spawn(move || {
            loop {
                while let Ok(action) = wakeup_receiver.recv() {
                    match action {
                        StateAction::Timer(event) => {
                            match event {
                                TimerEvent::HourChange => {
                                    if state.read().unwrap().fuel == 0 {
                                        println!("Povver Plant: Fuel is low");
                                        let balance = state.read().unwrap().balance.val();
                                        let fuel_price = econ_state.read().unwrap().fuel_price.val();
                                        let max_amount = balance / fuel_price;

                                        if max_amount >= 1.0 {
                                            let amount = ((max_amount / 10.0) + 1.0) as SimInt;
                                            println!("Povver Plant: Buying fuel for amount {amount}");
                                            signal_sender.send(PovverPlantSignals::BuyFuel(amount)).unwrap();
                                        }
                                    }
                                },
                                _ => ()
                            }
                        },
                        StateAction::Quit => {
                            break;
                        },
                        _ => ()
                    }
                }
            }
        })
    }
}