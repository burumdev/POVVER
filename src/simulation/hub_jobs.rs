use std::sync::Arc;
use crate::{
    logger::{Logger, LogLevel::*},
    simulation::{
        SimInt,
        hub::TheHub,
        sim_constants::*,
        hub_comms::*
    },
    economy::{
        products::ProductStock,
    },
};

#[derive(Debug, Clone)]
pub enum MinutelyJobKind {
    PPProducesEnergy(EnergyReceipt),
    FactoryProducesProduct(ProductionReceipt),
}

#[derive(Debug, Clone)]
pub struct MinutelyJob {
    pub kind: MinutelyJobKind,
    pub delay: SimInt,
    pub minute_created: SimInt,
}

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
    PPProductionCapIncrease,
}

#[derive(Debug, Clone)]
pub struct DailyJob {
    pub kind: DailyJobKind,
    pub delay: SimInt,
    pub day_created: SimInt,
}

impl TheHub {
    pub fn do_minutely_jobs(&mut self) {
        let this_minute = self.timer_state_ro.read().unwrap().date.minute;

        let mut due_jobs = Vec::new();
        self.minutely_jobs
            .retain_mut(|job| {
                if (job.minute_created + job.delay) % 60 == this_minute {
                    due_jobs.push(job.clone());
                    return false;
                }

                true
            });

        for job in due_jobs.drain(..) {
            match job.kind {
                MinutelyJobKind::PPProducesEnergy(receipt) => {
                    self.pp_energy_to_factory(receipt);
                }
                MinutelyJobKind::FactoryProducesProduct(receipt) => {
                    self.factory_produce(receipt);
                }
            }
        }
    }

    pub fn do_hourly_jobs(&mut self) {
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
                DailyJobKind::PPProductionCapIncrease => {
                    self.increase_pp_prod_cap();
                }
            }
        }
    }

    pub fn transfer_fuel_to_pp(&self, receipt: FuelReceipt) {
        self.log_ui_console(format!("Transfering {} fuel to Povver Plant.", receipt.units), Info);

        let mut pp = self.povver_plant_state.write().unwrap();
        pp.fuel += receipt.units;
        pp.is_awaiting_fuel = false;
        self.comms.hub_to_pp(Arc::new(HubPPSignal::FuelTransfered(receipt)));
    }

    pub fn increase_pp_fuel_cap(&self) {
        self.log_ui_console(format!("Increasing povver plant fuel capacity by {PP_FUEL_CAPACITY_INCREASE}."), Info);
        self.povver_plant_state.write().unwrap().fuel_capacity += PP_FUEL_CAPACITY_INCREASE;
        self.povver_plant_state.write().unwrap().is_awaiting_fuel_capacity = false;
        self.comms.hub_to_pp(Arc::new(HubPPSignal::FuelCapacityIncreased));
    }

    pub fn increase_pp_prod_cap(&self) {
        self.log_ui_console(format!("Increasing povver plant production capacity by {}.", PP_PRODUCTION_CAPACITY_INCREASE), Info);
        self.povver_plant_state.write().unwrap().production_capacity.inc(PP_PRODUCTION_CAPACITY_INCREASE);
        self.povver_plant_state.write().unwrap().is_awaiting_production_capacity = false;
        self.comms.hub_to_pp(Arc::new(HubPPSignal::ProductionCapacityIncreased));
    }

    pub fn pp_energy_to_factory(&self, receipt: EnergyReceipt) {
        let fid = receipt.factory_id;

        if let Some(factory) = self.get_factory_state(fid) {
            factory.write().unwrap().available_energy.inc(receipt.units);
            self.povver_plant_state.write().unwrap().balance.inc(receipt.total_price);
            let fuel_needed = receipt.units / PP_ENERGY_PER_FUEL;
            self.povver_plant_state.write().unwrap().fuel -= fuel_needed;

            self.log_ui_console(format!("Energy of {} units transfered to Factory No. {} from Povver Plant.", receipt.units, fid), Info);

            self.comms.hub_to_factory(fid, Arc::new(HubFactorySignal::EnergyTransfered(receipt.clone())));
            self.comms.hub_to_pp(Arc::new(HubPPSignal::EnergyTransfered(receipt)));
        } else {
            self.log_console(format!("Factory No. {} is not found. PP energy transfer canceled.", fid), Error);
        }
    }

    pub fn factory_produce(&mut self, receipt: ProductionReceipt) {
        let fid = receipt.factory_id;

        if let Some(factory) = self.get_factory_state(fid) {
            let units = receipt.demand.units;
            factory.write().unwrap().product_stocks.push(ProductStock {
                product: receipt.demand.product,
                units,
                unit_production_cost: receipt.price_per_unit,
            });
            self.log_ui_console(format!("Factory No. {} produced {} {}", fid, units, receipt.demand.product.name), Info);
            self.comms.hub_to_factory(fid, Arc::new(HubFactorySignal::ProductionComplete(receipt)));
        } else {
            self.log_console(format!("Factory No. {} is not found. {} production canceled.", fid, receipt.demand.product.name), Error);
        }
    }
}
