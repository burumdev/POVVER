use std::sync::Arc;
use crate::{
    logger::{Logger, LogLevel::*},
    simulation::{
        SimInt,
        hub::TheHub,
        hub_constants::*,
        hub_comms::*
    },
};

#[derive(Debug, Clone)]
pub enum HourlyJobKind {
    PPBoughtFuel(FuelReceipt),
}

#[derive(Debug, Clone)]
pub struct HourlyJob {
    pub kind: HourlyJobKind,
    pub delay: SimInt,
    pub hour_created: SimInt,
}

#[derive(Debug, Clone)]
pub enum DailyJobKind {
    PPFuelCapIncrease,
}

#[derive(Debug, Clone)]
pub struct DailyJob {
    pub kind: DailyJobKind,
    pub delay: SimInt,
    pub day_created: SimInt,
}

impl TheHub {
    pub fn do_hourly_jobs(&mut self) {
        self.log_console(format!("processing {} hourly jobs: {:?}", self.hourly_jobs.len(), self.hourly_jobs), Info);
        let this_hour = self.timer_state_ro.read().unwrap().date.hour;

        let mut due_jobs = Vec::new();
        self.hourly_jobs
            .retain_mut(|job| {
                if (job.hour_created + job.delay) % 23 == this_hour {
                    due_jobs.push(job.clone());
                    return false;
                }

                true
            });

        for job in due_jobs.drain(..) {
            match job.kind {
                HourlyJobKind::PPBoughtFuel(receipt) => {
                    self.transfer_fuel_to_pp(receipt);
                }
            }
        }
    }

    pub fn do_daily_jobs(&mut self) {
        self.log_console(format!("processing {} daily jobs: {:?}", self.daily_jobs.len(), self.daily_jobs), Info);
        let today = self.timer_state_ro.read().unwrap().date.day;

        let mut due_jobs = Vec::new();
        self.daily_jobs
            .retain_mut(|job| {
                if (job.day_created + job.delay) % 30 == today {
                    due_jobs.push(job.clone());
                    return false;
                }

                true
            });

        for job in due_jobs.drain(..) {
            match job.kind {
                DailyJobKind::PPFuelCapIncrease => {
                    self.increase_pp_fuel_cap();
                }
            }
        }
    }

    pub fn transfer_fuel_to_pp(&self, receipt: FuelReceipt) {
        self.log_ui_console(format!("Transfering {} fuel to Povver Plant.", receipt.amount), Info);

        let mut pp = self.povver_plant_state.write().unwrap();
        pp.fuel += receipt.amount;
        pp.is_awaiting_fuel = false;
        self.comms.hub_to_pp(Arc::new(HubPPSignal::FuelTransfered(receipt)));
    }

    pub fn increase_pp_fuel_cap(&self) {
        self.log_ui_console(format!("Increasing povver plant fuel capacity by {PP_FUEL_CAPACITY_INCREASE}."), Info);
        self.povver_plant_state.write().unwrap().fuel_capacity += PP_FUEL_CAPACITY_INCREASE;
        self.comms.hub_to_pp(Arc::new(HubPPSignal::FuelCapacityIncreased));
    }
}
