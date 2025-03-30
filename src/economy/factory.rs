use std::{
    thread,
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::sync::broadcast as tokio_broadcast;

use crate::{
    app_state::{FactoryStateData, EconomyStateData},
    simulation::{
        SimInt,
        SimFlo,
        StateAction,
        hub_comms::*,
        speed::Speed,
        Percentage
    },
    economy::{
        economy_types::{EnergyUnit, Money, ProductDemand},
        products::Product,
    },
    logger::{LogMessage, Logger, LogLevel::*},
    utils_data::ReadOnlyRwLock,
    utils_traits::AsFactor,
};

struct ProductionRun {
    demand: ProductDemand,
    cost: Money,
    energy_needed: EnergyUnit,
}

pub struct Factory {
    state_ro: ReadOnlyRwLock<FactoryStateData>,
    econ_state_ro: ReadOnlyRwLock<EconomyStateData>,
    ui_log_sender: tokio_broadcast::Sender<LogMessage>,
    wakeup_receiver: tokio_broadcast::Receiver<StateAction>,
    hub_broadcast_receiver: BroadcastDynReceiver,
    dynamic_sender: BroadcastDynSender,
    dynamic_receiver: DynamicReceiver,
    production_runs: Vec<ProductionRun>,
    last_hundred_energy_purchases: Vec<EnergyReceipt>,
    product_demand_sell_threshold: Percentage,
    profit_margin: Percentage,
}

impl Factory {
    pub fn new(
        state_ro: ReadOnlyRwLock<FactoryStateData>,
        econ_state_ro: ReadOnlyRwLock<EconomyStateData>,
        ui_log_sender: tokio_broadcast::Sender<LogMessage>,
        wakeup_receiver: tokio_broadcast::Receiver<StateAction>,
        hub_broadcast_receiver: tokio_broadcast::Receiver<DynamicSignal>,
        dynamic_sender: BroadcastDynSender,
        dynamic_receiver: DynamicReceiver,
    ) -> Self {
        Self {
            state_ro,
            econ_state_ro,
            ui_log_sender,
            wakeup_receiver,
            hub_broadcast_receiver,
            dynamic_sender,
            dynamic_receiver,
            production_runs: Vec::new(),
            last_hundred_energy_purchases: Vec::new(),
            product_demand_sell_threshold: Percentage::new(0.0),
            profit_margin: Percentage::new(20.0),
        }
    }
}

impl Factory {
    fn product_index_in_prod_run(&self, product: &Product) -> Option<usize> {
        self.production_runs.iter().position(|run| run.demand.product == product)
    }

    fn product_index_in_stock(&self, product: &Product) -> Option<usize> {
        self.state_ro.read().unwrap().product_stocks.iter().position(|stock| stock.product == product)
    }

    fn maybe_produce_goods(&mut self) {
        let (factory_id, balance, producable_demands) = {
            let econ_state_ro = self.econ_state_ro.read().unwrap();
            let state_ro = self.state_ro.read().unwrap();
            let producable_demands = econ_state_ro
                .product_demands
                .iter()
                .copied()
                .filter(
                    |demand|
                        demand.product.industry == state_ro.industry && state_ro.product_portfolio.contains(&demand.product) &&
                        self.product_index_in_prod_run(demand.product).is_none() && self.product_index_in_stock(demand.product).is_none()
                )
                .collect::<Vec<_>>();

            (
                state_ro.id,
                state_ro.balance,
                producable_demands,
            )
        };

        if producable_demands.len() > 0 {
            for demand in producable_demands {
                let product = &demand.product;

                let unit_cost_ex_energy = product.get_unit_cost_excl_energy();
                let units = demand.as_units();
                let total_cost_ex_energy = unit_cost_ex_energy * units as SimFlo;

                if total_cost_ex_energy <= balance * 0.75 {
                    let energy_needed = demand.calculate_energy_need();
                    let energy_demand = FactoryEnergyDemand {
                        factory_id,
                        energy_needed,
                    };

                    self.dynamic_sender.send(Arc::new(
                        FactoryHubSignal::EnergyDemand(
                            energy_demand,
                        ))
                    ).unwrap();

                    self.production_runs.push(ProductionRun {
                        demand,
                        cost: total_cost_ex_energy,
                        energy_needed
                    })
                }
            }
        } else {
            self.log_console("No demands are producable".to_string(), Info);
        }
    }

    fn evaluate_pp_energy_offer(&mut self, offer: &PPEnergyOffer) {
        let balance = self.state_ro.read().unwrap().balance;

        if !self.production_runs.is_empty() {
            // TODO: A more sophisticated algo to evaluate the price here might be better option.
            let prun = self.production_runs.last_mut().unwrap();
            let energy_cost = offer.price_per_unit * offer.units.val() as SimFlo;
            let remaining_budget = balance - (prun.cost + energy_cost);
            if remaining_budget.val() > 0.0 {
                prun.cost.inc(energy_cost.val());

                self.dynamic_sender.send(Arc::new(FactorySignal::AcceptPPEnergyOffer(*offer))).unwrap();
            } else {
                self.dynamic_sender.send(Arc::new(FactorySignal::RejectPPEnergyOffer(*offer))).unwrap();
            }
        }
    }

    fn energy_received(&mut self) {
        let (balance, energy_available) = {
            let state_ro = self.state_ro.read().unwrap();
            (state_ro.balance, state_ro.available_energy)
        };

        if let Some(index) = self.production_runs.iter().rposition(|run| {
            run.energy_needed <= energy_available && run.cost <= balance
        }) {
            let run = &self.production_runs[index];
            let unit_cost = Money::new(run.cost.val() / run.demand.units as SimFlo);
            self.produce_product_demand(run.demand, unit_cost);
        }
    }

    fn produce_product_demand(&mut self, demand: ProductDemand, unit_cost: Money) {
        self.dynamic_sender.send(
            Arc::new(FactoryHubSignal::ProducingProductDemand(demand, unit_cost))
        ).unwrap();
    }

    fn maybe_sell_goods(&self) {
        let state_ro = self.state_ro.read().unwrap();
        let econ_state_ro = self.econ_state_ro.read().unwrap();
        state_ro.product_stocks.iter().for_each(|stock| {
            if let Some(index) = econ_state_ro.product_demands.iter()
                .position(|demand|
                    demand.product == stock.product && demand.percent.val() > self.product_demand_sell_threshold.val()
                ) {
                    let unit_price = stock.unit_production_cost + stock.unit_production_cost * self.profit_margin.as_factor();
                    self.dynamic_sender.send(Arc::new(FactoryHubSignal::SellingProduct(index, unit_price))).unwrap();
                }
        })
    }

    fn production_complete(&mut self, demand: &ProductDemand) {
        let demand_index = self.production_runs.iter().position(|run| run.demand == *demand);
        if let Some(remove_index) = demand_index {
            self.production_runs.remove(remove_index);
            self.maybe_sell_goods();
        }
    }
}

impl Factory {
    pub fn start(me: Arc<Mutex<Self>>) -> thread::JoinHandle<()> {
        let (
            my_id,
            state_ro,
            mut wakeup_receiver,
            mut hub_broadcast_receiver,
            dynamic_receiver
        ) = {
            let me_lock = me.lock().unwrap();
            (
                me_lock.state_ro.read().unwrap().id,
                ReadOnlyRwLock::clone(&me_lock.state_ro),
                me_lock.wakeup_receiver.resubscribe(),
                me_lock.hub_broadcast_receiver.resubscribe(),
                me_lock.dynamic_receiver.clone(),
            )
        };

        thread::Builder::new().name("POVVER_F".to_string() + &my_id.to_string()).spawn(move || {
            let mut sleeptime = Speed::NORMAL.get_tick_duration() / 2;
            'outer: loop {
                if let Ok(signal) = hub_broadcast_receiver.try_recv() {
                    let signal_any = signal.as_any();
                    match signal_any {
                        s if s.is::<FactoryEnergyDemand>() => {
                            if let Some(demand) = signal_any.downcast_ref::<FactoryEnergyDemand>() {
                                if demand.factory_id != my_id {
                                    //TODO
                                    me.lock().unwrap().log_console(format!("Got message: {:?} is from another guy :)", signal), Critical);
                                    // MAYBE SELL SOME LEFTOVER ENERGY TO THE FACTORY IN NEED
                                } else {
                                    //TODO
                                    me.lock().unwrap().log_console(format!("Got message: {:?} is from me haha :)", signal), Critical);
                                }
                            }
                        },
                        _ => {
                            //TODO
                            me.lock().unwrap().log_console(format!("Got message: {:?}. But is not an energy demand?", signal), Critical);
                        }
                    }
                }

                if let Ok(signal) = dynamic_receiver.try_recv() {
                    let signal_any = signal.as_any();
                    match signal_any {
                        s if s.is::<PPEnergyOffer>() => {
                            if let Some(offer) = signal_any.downcast_ref::<PPEnergyOffer>() {
                                let mut me_lock = me.lock().unwrap();
                                me_lock.log_ui_console(format!("Got energy offer from PP: {} units.", offer.units.val()), Info);
                                me_lock.evaluate_pp_energy_offer(offer);
                            }
                        },
                        s if s.is::<HubFactorySignal>() => {
                            if let Some(signal_from_hub) = signal_any.downcast_ref::<HubFactorySignal>() {
                                let mut me_lock = me.lock().unwrap();
                                match signal_from_hub {
                                    HubFactorySignal::EnergyTransfered(receipt) => {
                                        me_lock.log_ui_console(format!("{} units of energy received.", receipt.units.val()), Info);
                                        me_lock.last_hundred_energy_purchases.push(receipt.clone());
                                        me_lock.energy_received();
                                    }
                                    HubFactorySignal::ProductionComplete(demand) => {
                                        me_lock.production_complete(demand);
                                    }
                                }
                            }
                        },
                        _ => {
                            //TODO
                            me.lock().unwrap().log_console(format!("Got dynamic message: {:?}. But is not an energy offer?", signal), Critical);
                        }
                    }
                }

                if let Ok(action) = wakeup_receiver.try_recv() {
                    if !state_ro.read().unwrap().is_bankrupt {
                        match action {
                            StateAction::Timer(event) => {
                                if event.at_least_hour() {
                                    //TODO: Hourly factory errands
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
                                break 'outer;
                            }
                            _ => ()
                        }
                    } else { // Factory is BANKRUPT!
                        //TODO
                        me.lock().unwrap().log_ui_console("Gone belly up! We're bankrupt! Pivoting to ball bearing production ASAP!".to_string(), Critical);
                    }
                }
                thread::sleep(Duration::from_millis(sleeptime));
            }
        }).unwrap()
    }
}

impl Logger for Factory {
    fn get_log_prefix(&self) -> String {
        format!("Factory No. {}", self.state_ro.read().unwrap().id)
    }
    fn get_message_source(&self) -> MessageEntity {
        MessageEntity::Factory(self.state_ro.read().unwrap().id as SimInt)
    }
    fn get_log_sender(&self) -> &tokio_broadcast::Sender<LogMessage> {
        &self.ui_log_sender
    }
}
