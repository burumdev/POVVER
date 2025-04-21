use std::sync::Arc;
use std::thread;

use crate::{
    logger::{Logger, LogLevel::*},
    simulation::{
        SimInt,
        SimFlo,
        hub::TheHub,
        sim_constants::*,
        hub_jobs::*,
        hub_comms::*,
        Percentage
    },
    economy::{
        economy_types::ProductDemand,
    },
    utils_traits::{AsFactor, HundredPercentable},
};

impl TheHub {
    pub fn pp_buys_fuel(&mut self, amount: SimInt) {
        // Bureaucreaueautic delay
        thread::sleep(self.activity_delay_duration() * 2);

        let price = self.econ_state.read().unwrap().fuel_price;
        let fee = price.val() * amount as SimFlo;

        let transaction_successful =
            self.povver_plant_state.write().unwrap()
                .balance.dec(fee);

        if transaction_successful {
            let delay = (amount as SimFlo / 10.0).floor() as SimInt;
            let date = self.timer_state_ro.read().unwrap().date.clone();
            let receipt = FuelReceipt {
                units: amount, price_per_unit: price.val(),
                date,
                total_price: fee.val(),
            };

            if delay == 0 {
                self.transfer_fuel_to_pp(receipt);
            } else {
                self.hourly_jobs.push(
                    HourlyJob {
                        kind: HourlyJobKind::PPBoughtFuel(receipt),
                        delay,
                        timestamp: self.timer_state_ro.read().unwrap().timestamp,
                    }
                );
                self.log_ui_console(format!("PP bought fuel for amount {amount}. ETA is {delay} hours."), Info);
                self.povver_plant_state.write().unwrap().is_awaiting_fuel = true;
            }
        } else {
            self.log_ui_console(format!("PP couldn't pay for fuel amount {amount} for the price of {fee}. Transaction canceled."), Warning);
        }
    }

    pub fn pp_increases_fuel_capacity(&mut self) {
        // Bureaucreaueautic delay
        thread::sleep(self.activity_delay_duration() * 3);

        let transaction_successful =
            self.povver_plant_state.write().unwrap()
                .balance.dec(PP_FUEL_CAPACITY_INCREASE_COST.val());

        if transaction_successful {
            self.povver_plant_state.write().unwrap().is_awaiting_fuel_capacity = true;
            let delay = 1;
            self.daily_jobs.push(DailyJob {
                kind: DailyJobKind::PPFuelCapIncrease,
                delay,
                timestamp: self.timer_state_ro.read().unwrap().timestamp,
            });
            self.log_ui_console(format!("PP is upgrading it's fuel capacity. ETA is {delay} days."), Info);
            println!();
        } else {
            self.log_ui_console("PP couldn't pay for fuel capacity increase. Upgrade canceled.".to_string(), Critical);
        }
    }

    pub fn pp_increases_production_capacity(&mut self) {
        // Bureaucreaueautic delay
        thread::sleep(self.activity_delay_duration() * 5);

        let transaction_successful =
            self.povver_plant_state.write().unwrap()
                .balance.dec(PP_PRODUCTION_CAPACITY_INCREASE_COST.val());

        if transaction_successful {
            self.povver_plant_state.write().unwrap().is_awaiting_production_capacity = true;
            let delay = 3;
            self.daily_jobs.push(DailyJob {
                kind: DailyJobKind::PPProductionCapIncrease,
                delay,
                timestamp: self.timer_state_ro.read().unwrap().timestamp,
            });
            self.log_ui_console(format!("PP is upgrading it's production capacity. ETA is {delay} days."), Info);
        } else {
            self.log_ui_console("PP couldn't pay for production capacity increase. Upgrade canceled.".to_string(), Critical);
        }
    }

    pub fn pp_produces_energy(&mut self, offer: &PPEnergyOffer) {
        // Bureaucreaueautic delay
        thread::sleep(self.activity_delay_duration());

        let fid = offer.to_factory_id;
        if let Some(factory) = self.get_factory_state(fid) {
            let fee = offer.price_per_unit * offer.units as SimFlo;
            if !factory.write().unwrap().balance.dec(fee.val()) {
                factory.write().unwrap().is_bankrupt = true;

                self.log_ui_console(format!("Factory No. {} has gone bankrupt. I'm the hub. I don't go bankrupt.", fid), Critical);

                return;
            }

            let date = self.timer_state_ro.read().unwrap().date.clone();
            let receipt = EnergyReceipt {
                units: offer.units,
                price_per_unit: offer.price_per_unit.val(),
                date,
                factory_id: fid,
                total_price: fee.val(),
            };

            let delay = offer.units / 100;
            if delay == 0 {
                self.pp_energy_to_factory(receipt);
            } else {
                self.minutely_jobs.push(MinutelyJob {
                    kind: MinutelyJobKind::PPProducesEnergy(receipt),
                    delay,
                    timestamp: self.timer_state_ro.read().unwrap().timestamp,
                });
                self.log_ui_console(format!("PP is producing {} units of energy for factory No. {}. ETA is {} minutes.", offer.units, fid, delay), Info);
            }
        } else {
            self.log_console(format!("Factory No. {} is not found. PP energy production canceled.", fid), Error);
        }
    }

    pub fn factory_needs_energy(&mut self, demand: &FactoryEnergyDemand) {
        // Bureaucreaueautic delay
        thread::sleep(self.activity_delay_duration());

        self.comms.send_signal_broadcast(Arc::new(*demand))
    }

    pub fn factory_will_produce(&mut self, fid: usize, demand: &ProductDemand, units: SimInt, unit_cost: SimFlo) {
        // Bureaucreaueautic delay
        thread::sleep(self.activity_delay_duration() * 2);

        let unit_cost_ex_energy = demand.product.get_unit_cost_excl_energy();
        let energy_cost = demand.product.unit_production_cost.energy;

        if let Some(factory) = self.get_factory_state(fid) {
            let available_energy = factory.read().unwrap().available_energy;
            let producable_units = (available_energy.val() / energy_cost).clamp(0, units);
            if producable_units < units {
                self.log_ui_console(format!("Factory No. {} has not enough energy to produce {} {}. Producing {} units instead.", fid, units, demand.product.name, producable_units), Warning);
            }

            let total_cost_ex_energy = unit_cost_ex_energy * producable_units as SimFlo;
            let transaction_successful = factory.write().unwrap().balance.dec(total_cost_ex_energy.val());
            if transaction_successful {
                //TODO: Turn this * 3 modifier into an efficiency metric that can be improved by factory investments
                // So it goes like 3..2..1.. BOOM! And factory produces stuff as fast as possible.
                let delay = (producable_units / demand.product.units_per_minute) * 3;
                let receipt = ProductionReceipt {
                    demand: demand.clone(),
                    units_produced: producable_units,
                    price_per_unit: unit_cost,
                    date: self.timer_state_ro.read().unwrap().date.clone(),
                    factory_id: fid,
                    total_price: total_cost_ex_energy.val(),
                };

                self.minutely_jobs.push(MinutelyJob {
                    kind: MinutelyJobKind::FactoryProducesProduct(receipt),
                    delay,
                    timestamp: self.timer_state_ro.read().unwrap().timestamp,
                });
            } else {
                factory.write().unwrap().is_bankrupt = true;
                self.log_ui_console(
                    format!(
                        "Factory No. {} has not enough money to produce {} {}. It's gone bankrupt.",
                        fid, units, demand.product.name
                    ), Critical);
                self.log_console(
                    format!(
                        "Unit cost excluding energy is {}. Total cost excl. energy is {}. Factory budget is {}",
                        unit_cost_ex_energy.val(), total_cost_ex_energy.val(), factory.read().unwrap().balance.val()
                    ), Critical);
            }
        }
    }

    pub fn factory_buys_solar_panels(&mut self, fid: usize, panels_count: usize) {
        // Bureaucreaueautic delay
        thread::sleep(self.activity_delay_duration() * 4);

        if let Some(factory) = self.get_factory_state(fid) {
            let fee = panels_count as SimFlo * SOLAR_PANEL_PRICE;
            let current_panels_count = factory.read().unwrap().solarpanels.len();
            let amount_purchasable = if current_panels_count + panels_count >= FACTORY_MAX_SOLAR_PANELS {
                FACTORY_MAX_SOLAR_PANELS - (current_panels_count - panels_count)
            } else {
                panels_count
            };

            if amount_purchasable > 0 {
                let transaction_successful = factory.write().unwrap().balance.dec(fee.val());
                if transaction_successful {
                    let delay = (panels_count as SimFlo / 6.0).ceil() as SimInt;

                    self.daily_jobs.push(DailyJob {
                        kind: DailyJobKind::FactoryBoughtSolarpanels(fid, panels_count),
                        delay,
                        timestamp: self.timer_state_ro.read().unwrap().timestamp,
                    });
                    self.log_ui_console(format!("Factory No. {} bought {} units of solar panels. ETA is {} day(s)", fid, panels_count, delay), Info);
                    factory.write().unwrap().is_awaiting_solarpanels = true;
                } else {
                    factory.write().unwrap().is_bankrupt = true;
                    self.log_ui_console(format!("Factory No. {} has gone bankrupt. It can't even pay for {} freaking solar panels!", fid, panels_count), Critical);
                }
            } else {
                self.log_ui_console(format!("Factory No. {} has reached it's solarpanel limit of {FACTORY_MAX_SOLAR_PANELS}. It can't buy another one!", fid), Warning);
            }
        } else {
            self.log_console(format!("Factory No. {} is not found. So it can't buy any solar panels now, can it?", fid), Error);
        }

    }

    pub fn factory_sells_product(&mut self, fid: usize, stock_index: usize, unit_price: SimFlo) {
        // Bureaucreaueautic delay
        thread::sleep(self.activity_delay_duration() * 1);

        if let Some(factory) = self.get_factory_state(fid) {
            if factory.read().unwrap().product_stocks.get(stock_index).is_some() {
                let mut fac = factory.write().unwrap();
                let stock = fac.product_stocks.remove(stock_index);
                if let Some(demand) = self.econ_state.write().unwrap().product_demands.iter_mut().find(|demand| demand.product == stock.product) {
                    // TODO: A more complicated code to determine how many to buy would be better.
                    // For now all units are bought at the price set by the factory.
                    let met_percent = Percentage::new(stock.units as SimFlo / demand.units as SimFlo * 100.0);
                    demand.demand_meet_percent = met_percent;
                    demand.units -= stock.units;
                    demand.percent.set(demand.percent.val() - (demand.percent.val() * met_percent.as_factor()));
                    let total_price = stock.units as SimFlo * unit_price;
                    fac.balance.inc(total_price.val());

                    self.log_ui_console(format!("Factory No. {} sold {} units of {} for a total price of {}.", fid, stock.units, stock.product.name, total_price.val()), Info);
                }
            } else {
                self.log_console(format!("factory_sells_product called with illegal stock index {}. Stock is: {:?}", stock_index, factory.read().unwrap().product_stocks), Error);
            }
        } else {
            self.log_console(format!("Factory No. {} is not found. Sale of product canceled.", fid), Error);
        }
    }
}
