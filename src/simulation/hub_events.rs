use std::sync::Arc;

use crate::{
    logger::{Logger, LogLevel::*},
    simulation::{
        SimInt,
        SimFlo,
        hub::TheHub,
        hub_constants::*,
        hub_types::*,
        hub_comms::*
    }
};

impl TheHub {
    pub fn pp_buys_fuel(&mut self, amount: SimInt) {
        let price = self.econ_state_ro.read().unwrap().fuel_price;
        let fee = price.val() * amount as SimFlo;

        let transaction_successful =
            self.povver_plant_state.write().unwrap()
                .balance.dec(fee);

        if transaction_successful {
            let delay = (amount as SimFlo / 5.0).floor() as SimInt;
            if delay == 0 {
                self.transfer_fuel_to_pp(amount);
            } else {
                let hour_created = self.timer_state_ro.read().unwrap().date.hour;
                self.hourly_jobs.push(
                    HourlyJob {
                        kind: HourlyJobKind::PPBoughtFuel(amount),
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
            self.daily_jobs.push(DailyJob {
                kind: DailyJobKind::PPFuelCapIncrease,
                delay: 5,
                day_created: self.timer_state_ro.read().unwrap().date.day,
            });
            self.log_ui_console("PP is upgrading it's fuel capacity. ETA is 5 days.".to_string(), Info);
            println!();
        } else {
            self.log_ui_console("PP couldn't pay for fuel capacity increase. Upgrade canceled.".to_string(), Critical);
        }
    }

    pub fn factory_needs_energy(&mut self, demand: FactoryEnergyDemand) {
        self.comms.send_signal_broadcast(Arc::new(demand))
    }
}
