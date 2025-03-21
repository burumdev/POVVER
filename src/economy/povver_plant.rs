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
    utils_traits::AsFactor,
    simulation::{
        StateAction,
        timer::TimerEvent,
        hub_comms::{PPHubSignal, HubPPSignal, MessageEntity, Broadcastable, FactoryEnergyDemand, PPEnergyOffer},
        SimInt,
        SimFlo,
        hub_constants::{PP_FUEL_CAPACITY_INCREASE_COST, PP_INIT_FUEL_BUY_THRESHOLD, PP_ENERGY_PER_FUEL},
        speed::Speed,
    },
    logger::{
        Logger,
        LogLevel::{Info, Warning, Critical},
        LogMessage,
    },
};
use crate::economy::economy_types::EnergyUnit;
use crate::simulation::hub_comms::FuelReceipt;
use crate::simulation::hub_comms::MessageEntity::PP;
use crate::simulation::hub_comms::PPHubSignal::EnergyOfferToFactory;
use crate::simulation::Percentage;

pub struct PovverPlant {
    profit_margin: Percentage,
    fuel_buy_threshold: SimInt,
    fuel_price_paid_per_unit_average: SimFlo,
    total_fuel_expenditure: SimFlo,
    last_ten_sales: SlidingWindow<Money>,
    state_ro: ReadOnlyRwLock<PovverPlantStateData>,
    econ_state_ro: ReadOnlyRwLock<EconomyStateData>,
    ui_log_sender: tokio_broadcast::Sender<LogMessage>,
    wakeup_receiver: tokio_broadcast::Receiver<StateAction>,
    pp_hub_sender: Sender<PPHubSignal>,
    hub_pp_receiver: Receiver<HubPPSignal>,
    hub_broadcast_receiver: tokio_broadcast::Receiver<Arc<dyn Broadcastable>>
}

impl PovverPlant {
    pub fn new(
        state_ro: ReadOnlyRwLock<PovverPlantStateData>,
        econ_state_ro: ReadOnlyRwLock<EconomyStateData>,
        ui_log_sender: tokio_broadcast::Sender<LogMessage>,
        wakeup_receiver: tokio_broadcast::Receiver<StateAction>,
        pp_hub_sender: Sender<PPHubSignal>,
        hub_pp_receiver: Receiver<HubPPSignal>,
        hub_broadcast_receiver: tokio_broadcast::Receiver<Arc<dyn Broadcastable>>
    ) -> Self {
        Self {
            profit_margin: Percentage::new(50.0),
            fuel_buy_threshold: PP_INIT_FUEL_BUY_THRESHOLD,
            fuel_price_paid_per_unit_average: 0.0,
            total_fuel_expenditure: 0.0,
            last_ten_sales: SlidingWindow::new(10),
            state_ro,
            econ_state_ro,
            ui_log_sender,
            wakeup_receiver,
            pp_hub_sender,
            hub_pp_receiver,
            hub_broadcast_receiver,
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
                        let amount = (((max_amount / 10.0) + 1.0) as SimInt).min(fuel_capacity);
                        if amount == fuel_capacity {
                            self.maybe_upgrade_fuel_capacity(balance);
                        }
                        self.log_ui_console(format!("Buying fuel for amount {amount}"), Info);
                        self.pp_hub_sender.send(PPHubSignal::BuyFuel(amount)).unwrap();
                    } else {
                        //TODO: Probably pp got bankrupt here
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

    fn update_price_paid_per_fuel_average(&mut self, receipt: FuelReceipt) {
        self.total_fuel_expenditure += receipt.amount as SimFlo * receipt.price_per_unit;
        self.fuel_price_paid_per_unit_average = self.total_fuel_expenditure / self.state_ro.read().unwrap().fuel as SimFlo;
    }

    fn maybe_upgrade_fuel_capacity(&mut self, balance: SimFlo) {
        if (balance / 4.0) > PP_FUEL_CAPACITY_INCREASE_COST.val() {
            self.pp_hub_sender.send(PPHubSignal::IncreaseFuelCapacity).unwrap();
        }
    }

    fn maybe_new_energy_offer(&mut self, demand: &FactoryEnergyDemand) -> Option<PPEnergyOffer> {
        let energy_per_fuel = PP_ENERGY_PER_FUEL;

        let (fuel, production_capacity) = {
            let state = self.state_ro.read().unwrap();
            (
                state.fuel_capacity,
                state.production_capacity,
            )
        };

        let fuel_needed = (demand.energy / energy_per_fuel).val();
        let producable = EnergyUnit::new((fuel * energy_per_fuel.val()).min(production_capacity.val()));

        if producable.val() == 0 {
            self.fuel_buy_threshold = fuel_needed;
            self.check_buy_fuel();
            return None;
        }

        let mut price_per_unit = Money::new(self.fuel_price_paid_per_unit_average);

        // We have both the production capacity and the fuel required
        // So we can produce all the demanded energy in one go and
        // Request maximum profit added on top.
        if producable >= demand.energy {
            price_per_unit.inc(price_per_unit.val() * self.profit_margin.as_factor());

            Some(PPEnergyOffer {
                price_per_unit,
                units: demand.energy,
                to_factory_id: demand.factory_id
            })
        } else {
            self.fuel_buy_threshold = fuel_needed;
            self.check_buy_fuel();
            // If we have production capacity shortcomings, this might be good time to try upgrading
            // production capacity and to a less extent fuel capacity to match it if we have the money.
            if production_capacity < demand.energy {
                self.maybe_upgrade_production_capacity();
            }

            // If our production falls short of demand, we can offer a lesser amount of energy to the
            // factory. With a discount in profit margin proportional to the deficit.
            let deficit = (producable - demand.energy).val().abs();
            let deficit_percent = Percentage::new((deficit / demand.energy.val()) as SimFlo * 100.0);

            let discounted_percent = self.profit_margin.val() - self.profit_margin.val() * deficit_percent.as_factor();

            price_per_unit.dec(price_per_unit.val() * discounted_percent.as_factor());

            Some(PPEnergyOffer {
                price_per_unit,
                units: producable,
                to_factory_id: demand.factory_id,
            })
        }
    }

    fn maybe_upgrade_production_capacity(&self) {
        //TODO
    }
}

impl PovverPlant {
    pub fn start(
        me: Arc<Mutex<Self>>,
    ) -> thread::JoinHandle<()> {
        let (state_ro, mut wakeup_receiver, hub_pp_receiver, mut hub_broadcast_receiver) = {
            let me_lock = me.lock().unwrap();
            (
                ReadOnlyRwLock::clone(&me_lock.state_ro),
                me_lock.wakeup_receiver.resubscribe(),
                me_lock.hub_pp_receiver.clone(),
                me_lock.hub_broadcast_receiver.resubscribe(),
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
                                if let Some(offer) = me.lock().unwrap().maybe_new_energy_offer(&demand) {
                                    let me_lock = me.lock().unwrap();

                                    me_lock.log_ui_console(
                                        format!("Sending energy offer to factory no: {}, amount: {} and price per EU: {}",
                                                offer.to_factory_id,
                                                offer.units.val(),
                                                offer.price_per_unit.val()
                                        ), Info
                                    );
                                    me_lock.pp_hub_sender.send(EnergyOfferToFactory(offer)).unwrap();
                                }
                            }
                        }
                        _ => ()
                    }
                }
                if let Ok(signal) = hub_pp_receiver.try_recv() {
                    match signal {
                        HubPPSignal::FuelTransfered(receipt) => {
                            me.lock().unwrap().update_price_paid_per_fuel_average(receipt);
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