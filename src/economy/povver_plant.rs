use std::{
    thread,
    sync::{Arc, Mutex},
};
use std::time::Duration;
use tokio::sync::broadcast as tokio_broadcast;

use crate::{
    app_state::{EconomyStateData, PovverPlantStateData},
    economy::economy_types::{Money, EnergyUnit},
    utils_data::{SlidingWindow, ReadOnlyRwLock},
    utils_traits::AsFactor,
    simulation::{
        StateAction,
        Percentage,
        timer::TimerEvent,
        hub_comms::*,
        SimInt,
        SimFlo,
        hub_constants::{PP_FUEL_CAPACITY_INCREASE_COST, PP_INIT_FUEL_BUY_THRESHOLD, PP_ENERGY_PER_FUEL},
        speed::Speed,
    },
    logger::{
        Logger,
        LogLevel::{Info, Warning, Critical, Error},
        LogMessage,
    },
};

pub struct PovverPlant {
    profit_margin: Percentage,
    fuel_buy_threshold: SimInt,
    fuel_price_paid_per_unit_average: SimFlo,
    total_fuel_expenditure: SimFlo,
    pending_energy_offers: Vec<PPEnergyOffer>,
    last_hundred_sales: SlidingWindow<EnergyReceipt>,
    state_ro: ReadOnlyRwLock<PovverPlantStateData>,
    econ_state_ro: ReadOnlyRwLock<EconomyStateData>,
    ui_log_sender: tokio_broadcast::Sender<LogMessage>,
    wakeup_receiver: tokio_broadcast::Receiver<StateAction>,
    dynamic_channel: DynamicChannel,
    hub_broadcast_receiver: BroadcastDynReceiver,
    to_factory_senders: Vec<DynamicSender>,
    from_factory_receivers: Vec<BroadcastDynReceiver>,
}

impl PovverPlant {
    pub fn new(
        state_ro: ReadOnlyRwLock<PovverPlantStateData>,
        econ_state_ro: ReadOnlyRwLock<EconomyStateData>,
        ui_log_sender: tokio_broadcast::Sender<LogMessage>,
        wakeup_receiver: tokio_broadcast::Receiver<StateAction>,
        dynamic_channel: DynamicChannel,
        hub_broadcast_receiver: tokio_broadcast::Receiver<DynamicSignal>,
        to_factory_senders: Vec<DynamicSender>,
        from_factory_receivers: Vec<BroadcastDynReceiver>,
    ) -> Self {
        let fuel_price = econ_state_ro.read().unwrap().fuel_price;
        let fuel_price_paid_per_unit_average = fuel_price.val();
        let total_fuel_expenditure = fuel_price.val() * state_ro.read().unwrap().fuel as SimFlo;

        Self {
            profit_margin: Percentage::new(50.0),
            fuel_buy_threshold: PP_INIT_FUEL_BUY_THRESHOLD,
            fuel_price_paid_per_unit_average,
            total_fuel_expenditure,
            pending_energy_offers: Vec::new(),
            last_hundred_sales: SlidingWindow::new(100),
            state_ro,
            econ_state_ro,
            ui_log_sender,
            wakeup_receiver,
            dynamic_channel,
            hub_broadcast_receiver,
            to_factory_senders,
            from_factory_receivers,
        }
    }
}

impl PovverPlant {
    fn get_dynamic_sender(&self) -> &DynamicSender {
        &self.dynamic_channel.0
    }

    fn get_dynamic_receiver(&self) -> &DynamicReceiver {
        &self.dynamic_channel.1
    }

    fn get_to_factory_sender_by_id(&self, factory_id: usize) -> &DynamicSender {
        &self.to_factory_senders[factory_id]
    }

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
                        self.get_dynamic_sender().send(Arc::new(PPHubSignal::BuyFuel(amount))).unwrap();
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

    fn update_price_paid_per_fuel_average(&mut self, receipt: &FuelReceipt) {
        self.total_fuel_expenditure += receipt.units as SimFlo * receipt.price_per_unit;
        self.fuel_price_paid_per_unit_average = self.total_fuel_expenditure / self.state_ro.read().unwrap().fuel as SimFlo;
    }

    fn maybe_upgrade_fuel_capacity(&mut self, balance: SimFlo) {
        if (balance / 4.0) > PP_FUEL_CAPACITY_INCREASE_COST.val() {
            self.get_dynamic_sender().send(Arc::new(PPHubSignal::IncreaseFuelCapacity)).unwrap();
        }
    }

    fn maybe_new_energy_offer(&mut self, demand: &FactoryEnergyDemand) {
        if let Some(_) = self.pending_energy_offers.iter().position(|of| of.to_factory_id == demand.factory_id) {
            self.log_console(format!("Energy demand from factory No. {} is already pending.", demand.factory_id), Info);
            return;
        }
        let energy_per_fuel = PP_ENERGY_PER_FUEL;
        let energy_needed = demand.energy_needed;
        let (fuel, production_capacity) = {
            let state = self.state_ro.read().unwrap();
            (
                state.fuel,
                state.production_capacity,
            )
        };

        let fuel_needed = (energy_needed / energy_per_fuel).val();
        let producable = EnergyUnit::new((fuel * energy_per_fuel.val()).min(production_capacity.val()));

        // We have ZERO energy production portential.
        // What are we gonna do?
        // TODO: handle this situation and maybe declare bankruptcy?
        if producable.val() == 0 {
            self.fuel_buy_threshold = fuel_needed;
            self.check_buy_fuel();

            return;
        }

        let mut price_per_unit = Money::new(self.fuel_price_paid_per_unit_average / PP_ENERGY_PER_FUEL.val() as SimFlo);

        let mut offer = PPEnergyOffer {
            to_factory_id: demand.factory_id,
            ..PPEnergyOffer::default()
        };

        // We have both the production capacity and the fuel required
        // So we can produce all the demanded energy in one go and
        // Request maximum profit added on top.
        if producable >= energy_needed {
            offer.units = energy_needed;
            price_per_unit.inc(price_per_unit.val() * self.profit_margin.as_factor());

            self.log_ui_console(
                format!("Sending FULL energy offer to factory no: {}, amount: {} and price per EU: {}",
                    offer.to_factory_id,
                    offer.units.val(),
                    price_per_unit.val(),
                ), Info
            );
        } else {
            offer.units = producable;

            if fuel < fuel_needed {
                self.fuel_buy_threshold = fuel_needed;
                self.check_buy_fuel();
            }
            // If we have production capacity shortcomings, this might be good time to try upgrading
            // production capacity and to a less extent fuel capacity to match it if we have the money.
            if production_capacity < energy_needed {
                self.maybe_upgrade_production_capacity();
            }

            // If our production falls short of demand, we can offer a lesser amount of energy to the
            // factory. With a discount in profit margin proportional to the deficit.
            let deficit = (energy_needed - producable).val();
            let deficit_percent = Percentage::new((deficit as SimFlo / energy_needed.val() as SimFlo) * 100.0);

            let discounted_percent = self.profit_margin.val() - self.profit_margin.val() * deficit_percent.as_factor();

            price_per_unit.inc(price_per_unit.val() * discounted_percent.as_factor());

            self.log_ui_console(
                format!("Sending partial energy offer to factory no: {}, amount: {} and price per EU: {}",
                    offer.to_factory_id,
                    offer.units.val(),
                    price_per_unit.val(),
                ), Info
            );
        }

        offer.price_per_unit = price_per_unit;

        self.pending_energy_offers.push(offer);
        self.get_to_factory_sender_by_id(offer.to_factory_id).send(Arc::new(offer)).unwrap();
    }

    fn maybe_upgrade_production_capacity(&self) {
        //TODO
    }

    fn process_factory_order(&mut self, offer: &PPEnergyOffer) {
        let index = self.pending_energy_offers.iter().position(|of| of.to_factory_id == offer.to_factory_id);
        if let Some(index) = index {
            let plucked_offer = self.pending_energy_offers.remove(index);
            self.get_dynamic_sender().send(Arc::new(PPHubSignal::EnergyToFactory(plucked_offer))).unwrap();
            self.log_ui_console(format!("Energy to factory No. {} is coming right up!", offer.to_factory_id), Info);
        } else {
            self.log_console(format!("Energy offer to process: {:?} could not be found in pending offers: {:?}", offer, self.pending_energy_offers), Error);
        }
    }

    fn remove_pending_offer(&mut self, offer: &PPEnergyOffer) {
        self.log_ui_console(format!("Removing energy offer. Factory No. {} can burn horse chesnut shells for energy.", offer.to_factory_id), Warning);
        self.pending_energy_offers.retain(|of| of.to_factory_id != offer.to_factory_id);
    }
}

impl PovverPlant {
    pub fn start(
        me: Arc<Mutex<Self>>,
    ) -> thread::JoinHandle<()> {
        let (
            state_ro,
            mut wakeup_receiver,
            dynamic_receiver,
            mut hub_broadcast_receiver,
            mut from_factory_dyn_receivers
        ) = {
            let me_lock = me.lock().unwrap();
            (
                ReadOnlyRwLock::clone(&me_lock.state_ro),
                me_lock.wakeup_receiver.resubscribe(),
                me_lock.get_dynamic_receiver().clone(),
                me_lock.hub_broadcast_receiver.resubscribe(),
                me_lock.from_factory_receivers.iter().map(|r| r.resubscribe()).collect::<Vec<BroadcastDynReceiver>>(),
            )
        };

        thread::Builder::new().name("POVVER_PLANT".to_string()).spawn(move || {
            let mut sleeptime = Speed::NORMAL.get_tick_duration() / 2;
            'outer: loop {
                if let Ok(signal) = hub_broadcast_receiver.try_recv() {
                    let signal_any = signal.as_any();
                    match signal_any {
                        s if s.is::<FactoryEnergyDemand>() => {
                            if let Some(demand) = signal_any.downcast_ref::<FactoryEnergyDemand>() {
                                me.lock().unwrap().maybe_new_energy_offer(&demand);
                            }
                        }
                        _ => ()
                    }
                }
                if let Ok(signal) = dynamic_receiver.try_recv() {
                    let signal_any = signal.as_any();
                    match signal_any {
                        s if s.is::<HubPPSignal>() => {
                            if let Some(signal_from_hub) = signal_any.downcast_ref::<HubPPSignal>() {
                                match signal_from_hub {
                                    HubPPSignal::FuelTransfered(receipt) => {
                                        me.lock().unwrap().update_price_paid_per_fuel_average(receipt);
                                    }
                                    HubPPSignal::EnergyTransfered(receipt) => {
                                        me.lock().unwrap().last_hundred_sales.add(receipt.clone());
                                        //TODO
                                        // Energy transfered. Let's do something about it!
                                    },
                                    HubPPSignal::FuelCapacityIncreased => {
                                        //TODO
                                        // Fuel capacity increased. Let's do something about it!
                                    },
                                }
                            }
                        }
                        _ => ()
                    }
                }
                from_factory_dyn_receivers.iter_mut().for_each(|receiver| {
                    while let Ok(signal) = receiver.try_recv() {
                        let signal_any = signal.as_any();
                        match signal_any {
                            s if s.is::<FactorySignal>() => {
                                if let Some(signal_from_factory) = signal_any.downcast_ref::<FactorySignal>() {
                                    match signal_from_factory {
                                        FactorySignal::AcceptPPEnergyOffer(offer) => {
                                            me.lock().unwrap().process_factory_order(offer);
                                        }
                                        FactorySignal::RejectPPEnergyOffer(offer) => {
                                            me.lock().unwrap().remove_pending_offer(offer);
                                        }
                                    }
                                }
                            }
                            _ => ()
                        }
                    }
                });
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
                                break 'outer;
                            }
                            _ => ()
                        }
                    } else { // PP is BANKRUPT!
                        //TODO
                        me.lock().unwrap().log_console("Gone belly up! We're bankrupt! Pivoting to potato salad production ASAP!".to_string(), Critical);
                    }
                }
                thread::sleep(Duration::from_millis(sleeptime));
            }
        }).unwrap()
    }
}

impl Logger for PovverPlant {
    fn get_log_prefix(&self) -> String {
        "Povver Plant".to_string()
    }
    fn get_message_source(&self) -> MessageEntity {
        MessageEntity::PP
    }
    fn get_log_sender(&self) -> &tokio_broadcast::Sender<LogMessage> {
        &self.ui_log_sender
    }
}