use std::sync::Arc;

use crate::{
    logger::{Logger, LogLevel::*},
    simulation::{
        SimInt,
        SimFlo,
        hub::TheHub,
        hub_constants::*,
        hub_jobs::*,
        hub_comms::*
    },
    economy::{
        products::ProductStock,
        economy_types::{Money, ProductDemand},
    },
    utils_traits::AsFactor,
};

impl TheHub {
    pub fn pp_buys_fuel(&mut self, amount: SimInt) {
        let price = self.econ_state.read().unwrap().fuel_price;
        let fee = price.val() * amount as SimFlo;

        let transaction_successful =
            self.povver_plant_state.write().unwrap()
                .balance.dec(fee);

        if transaction_successful {
            let delay = (amount as SimFlo / 15.0).floor() as SimInt;
            let date = self.timer_state_ro.read().unwrap().date.clone();
            let receipt = FuelReceipt {
                units: amount, price_per_unit: price.val(),
                date,
                total_price: fee.val(),
            };

            if delay == 0 {
                self.transfer_fuel_to_pp(receipt);
            } else {
                let hour_created = self.timer_state_ro.read().unwrap().date.hour;
                self.hourly_jobs.push(
                    HourlyJob {
                        kind: HourlyJobKind::PPBoughtFuel(receipt),
                        delay,
                        hour_created,
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
        let transaction_successful =
            self.povver_plant_state.write().unwrap()
                .balance.dec(PP_FUEL_CAPACITY_INCREASE_COST.val());

        if transaction_successful {
            let delay = 5;
            self.daily_jobs.push(DailyJob {
                kind: DailyJobKind::PPFuelCapIncrease,
                delay,
                day_created: self.timer_state_ro.read().unwrap().date.day,
            });
            self.log_ui_console(format!("PP is upgrading it's fuel capacity. ETA is {delay} days."), Info);
            println!();
        } else {
            self.log_ui_console("PP couldn't pay for fuel capacity increase. Upgrade canceled.".to_string(), Critical);
        }
    }

    pub fn pp_increases_production_capacity(&mut self) {
        let transaction_successful =
            self.povver_plant_state.write().unwrap()
                .balance.dec(PP_PRODUCTION_CAPACITY_INCREASE_COST.val());

        if transaction_successful {
            let delay = 7;
            self.daily_jobs.push(DailyJob {
                kind: DailyJobKind::PPProductionCapIncrease,
                delay,
                day_created: self.timer_state_ro.read().unwrap().date.day,
            });
            self.log_ui_console(format!("PP is upgrading it's production capacity. ETA is {delay} days."), Info);
            println!();
        } else {
            self.log_ui_console("PP couldn't pay for production capacity increase. Upgrade canceled.".to_string(), Critical);
        }
    }

    pub fn pp_produces_energy(&mut self, offer: &PPEnergyOffer) {
        let fid = offer.to_factory_id;
        if let Some(factory) = self.get_factory_state(fid) {
            let fee = offer.price_per_unit * offer.units.val() as SimFlo;
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

            let delay = offer.units.val() / 1000;
            let minute_created = receipt.date.minute;
            self.minutely_jobs.push(MinutelyJob {
                kind: MinutelyJobKind::PPProducesEnergy(receipt),
                delay,
                minute_created,
            });
            self.log_ui_console(format!("PP is producing {} units of energy for factory No. {}. ETA is {} minutes.", offer.units.val(), fid, delay), Info);

        } else {
            self.log_console(format!("Factory No. {} is not found. PP energy production canceled.", fid), Error);
        }
    }

    pub fn factory_needs_energy(&mut self, demand: &FactoryEnergyDemand) {
        self.comms.send_signal_broadcast(Arc::new(*demand))
    }

    pub fn factory_will_produce(&mut self, fid: usize, demand: &ProductDemand, unit_cost: &Money) {
        //TODO: This should be a timed job
        println!("Factory No. {} produces {} units for demand {:?}", fid, demand.units, demand);
        self.factory_produce(fid, demand, unit_cost);
    }

    pub fn factory_sells_product(&mut self, fid: usize, stock_index: usize, unit_price: Money) {
    }
}
