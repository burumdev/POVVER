use std::sync::Arc;
use crate::{
    logger::{Logger, LogLevel::*},
    simulation::{
        SimInt,
        SimFlo,
        hub::TheHub,
        sim_constants::*,
        hub_comms::*
    },
    economy::{
        products::ProductStock,
    },
};
use crate::economy::solarpanel::SolarPanel;
use crate::simulation::timer::TimerEvent;

#[derive(Debug, Clone)]
pub enum MinutelyJobKind {
    PPProducesEnergy(EnergyReceipt),
    FactoryProducesProduct(ProductionReceipt),
    FactoryProducedRenewableEnergy(usize, SimInt),
}

#[derive(Debug, Clone)]
pub struct MinutelyJob {
    pub kind: MinutelyJobKind,
    pub delay: SimInt,
    pub timestamp: u128,
}

#[derive(Debug, Clone)]
pub enum HourlyJobKind {
    PPBoughtFuel(FuelReceipt),
}

#[derive(Debug, Clone)]
pub struct HourlyJob {
    pub kind: HourlyJobKind,
    pub delay: SimInt,
    pub timestamp: u128,
}

#[derive(Debug, Clone)]
pub enum DailyJobKind {
    PPFuelCapIncrease,
    PPProductionCapIncrease,
    FactoryBoughtSolarpanels(usize, usize)
}

#[derive(Debug, Clone)]
pub struct DailyJob {
    pub kind: DailyJobKind,
    pub delay: SimInt,
    pub timestamp: u128,
}

impl TheHub {
    pub fn do_minutely_jobs(&mut self) {
        let now = self.timer_state_ro.read().unwrap().timestamp;

        let mut due_jobs = Vec::new();
        self.minutely_jobs
            .retain_mut(|job| {
                if (job.timestamp + job.delay as u128) <= now {
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
                MinutelyJobKind::FactoryProducedRenewableEnergy(fid, energy) => {
                    self.renewable_energy_to_factory(fid, energy);
                }
            }
        }
    }

    pub fn do_hourly_jobs(&mut self, event: &TimerEvent) {
        let now = self.timer_state_ro.read().unwrap().timestamp;

        let mut due_jobs = Vec::new();
        self.hourly_jobs
            .retain_mut(|job| {
                if (job.timestamp + (job.delay * 60) as u128) <= now {
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

        self.factories_renewable_produce_energy(event);
    }

    pub fn do_daily_jobs(&mut self) {
        let now = self.timer_state_ro.read().unwrap().timestamp;

        let mut due_jobs = Vec::new();
        self.daily_jobs
            .retain_mut(|job| {
                if (job.timestamp + (job.delay * 60 * 24) as u128) <= now {
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
                DailyJobKind::FactoryBoughtSolarpanels(fid, count) => {
                    self.solar_panel_to_factory(fid, count);
                }
            }
        }

        // If the factory doesn't use it's available energy in a day, it will be expired the next day.
        self.factories_energy_expired();
    }

    pub fn transfer_fuel_to_pp(&self, receipt: FuelReceipt) {
        self.log_ui_console(format!("Transfering {} fuel to Povver Plant.", receipt.units), Info);

        let mut pp = self.povver_plant_state.write().unwrap();
        pp.fuel += receipt.units.clamp(0, pp.fuel_capacity);
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
            let units = receipt.units_produced;
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

    pub fn factories_energy_expired(&self) {
        for factory in self.factories_state.read().unwrap().iter() {
            let mut fac_state = factory.write().unwrap();
            fac_state.available_energy.zero();
        }
    }

    pub fn factories_renewable_produce_energy(&mut self, event: &TimerEvent) {
        let sunshine = self.env_state_ro.read().unwrap().the_sun.brightness;
        for factory in self.factories_state.read().unwrap().iter() {
            let mut total_energy = 0;
            let mut solar_energy = 0;
            for solarpanel in factory.write().unwrap().solarpanels.iter_mut() {
                let energy = solarpanel.produce_energy(event, sunshine);
                solar_energy += energy;
            }

            //TODO: Wind turbines

            total_energy += solar_energy;
            if total_energy > 0 {
                let (fid, solar_count) = {
                    let state = factory.read().unwrap();
                    (
                        state.id,
                        state.solarpanels.len(),
                    )
                };
                let delay = (total_energy as SimFlo / 10.0).floor() as SimInt;
                if delay == 0 {
                    self.renewable_energy_to_factory(fid, total_energy);
                    //TODO: Report both solar and wind turbines once we have the turbines
                    self.log_ui_console(format!("Factory No. {} produced {} energy from {} solarpanels.", fid, total_energy, solar_count), Info);
                } else {
                    self.minutely_jobs.push(MinutelyJob {
                        kind: MinutelyJobKind::FactoryProducedRenewableEnergy(fid, total_energy),
                        delay,
                        timestamp: self.timer_state_ro.read().unwrap().timestamp,
                    });
                    //TODO: Report both solar and wind turbines once we have the turbines
                    self.log_ui_console(format!("Factory No. {} is producing {} energy from {} solarpanels. ETA is {} minutes.", fid, total_energy, solar_count, delay), Info);
                }
            }
        }
    }

    pub fn renewable_energy_to_factory(&self, fid: usize, energy: SimInt) {
        self.get_factory_state(fid).unwrap().write().unwrap().available_energy.inc(energy);
        self.comms.hub_to_factory(fid, Arc::new(HubFactorySignal::RenewableEnergyProduced));
    }

    pub fn solar_panel_to_factory(&self, fid: usize, count: usize) {
        if let Some(factory) = self.get_factory_state(fid) {
            let panels = vec![SolarPanel::new(); count];

            factory.write().unwrap().solarpanels.extend(panels);
            factory.write().unwrap().is_awaiting_solarpanels = false;
            self.log_ui_console(format!("Factory No. {} bought {} solarpanels. Watch'em go!", fid, count), Info);
        } else {
            self.log_console(format!("Factory No. {} is not found. So it can't buy any solar panels, period.", fid), Error);
        }
    }
}
