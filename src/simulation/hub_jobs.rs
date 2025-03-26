use std::ops::Index;
use std::sync::Arc;
use crate::{
    logger::{Logger, LogLevel::*},
    simulation::{
        SimInt,
        SimFlo,
        hub::TheHub,
        hub_constants::*,
        hub_comms::*
    },
    economy::products::ProductStock,
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
    PPProductionCapIncrease,
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
        self.comms.hub_to_pp(Arc::new(HubPPSignal::FuelCapacityIncreased));
    }

    pub fn increase_pp_prod_cap(&self) {
        self.log_ui_console(format!("Increasing povver plant production capacity by {}.", PP_PRODUCTION_CAPACITY_INCREASE.val()), Info);
        self.povver_plant_state.write().unwrap().production_capacity += PP_PRODUCTION_CAPACITY_INCREASE;
        self.comms.hub_to_pp(Arc::new(HubPPSignal::ProductionCapacityIncreased));
    }

    pub fn pp_energy_to_factory(&self, offer: &PPEnergyOffer) {
        let fid = offer.to_factory_id;
        let factories_state = self.factories_state.write().unwrap();
        let found_factory = factories_state.iter().find(|fac| fac.read().unwrap().id == fid);
        if let Some(factory) = found_factory {
            let fee = offer.price_per_unit * offer.units.val() as SimFlo;
            if !factory.write().unwrap().balance.dec(fee.val()) {
                factory.write().unwrap().is_bankrupt = true;

                self.log_ui_console(format!("Factory No. {} has gone bankrupt. I'm the hub. I don't go bankrupt.", fid), Critical);

                return;
            }

            factory.write().unwrap().available_energy.inc(offer.units);
            self.povver_plant_state.write().unwrap().balance.inc(fee.val());
            let fuel_needed = (offer.units / PP_ENERGY_PER_FUEL).val();
            self.povver_plant_state.write().unwrap().fuel -= fuel_needed;

            self.log_ui_console(format!("Energy of {} units transfered to Factory No. {} from Povver Plant.", offer.units.val(), fid), Info);

            let date = self.timer_state_ro.read().unwrap().date.clone();
            let receipt = EnergyReceipt {
                units: offer.units,
                price_per_unit: offer.price_per_unit.val(),
                date,
                factory_id: fid,
                total_price: fee.val(),
            };

            self.comms.hub_to_factory(Arc::new(HubFactorySignal::EnergyTransfered(receipt.clone())), fid);
            self.comms.hub_to_pp(Arc::new(HubPPSignal::EnergyTransfered(receipt)));
        } else {
            self.log_console(format!("Factory No. {} is not found. Energy transfer canceled.", fid), Error);
        }
    }

    pub fn factory_produce(&mut self, fid: usize, production: &FactoryProduction) {
        let demand = production.demand;
        let units = demand.as_units();
        let unit_cost_ex_energy = demand.as_units() * demand.product.get_unit_cost_excl_energy();
        let factories_state = self.factories_state.read().unwrap();
        let factory_state = factories_state.index(fid);
        let total_cost_ex_energy = unit_cost_ex_energy + units;

        let transaction_successful = factory_state.write().unwrap().balance.dec(total_cost_ex_energy.val());
        if transaction_successful {
            let energy_needed = production.demand.calculate_energy_need();
            let available_energy = factory_state.read().unwrap().available_energy;

            if available_energy >= energy_needed {
                factory_state.write().unwrap().product_stocks.push(ProductStock {
                    product: demand.product,
                    units
                });
            } else {
                self.log_ui_console(format!("Factory No. {} has not enough energy to produce {} {}", fid, units, demand.product.name), Critical);
            }

            self.log_ui_console(format!("Factory No. {} produced {} {}", fid, units, demand.product.name), Info);
        } else {
            factory_state.write().unwrap().is_bankrupt = true;
            self.log_ui_console(format!("Factory No. {} has not enough money to produce {} {}. It's gone bankrupt.", fid, units, demand.product.name), Critical);
        }
    }
}
