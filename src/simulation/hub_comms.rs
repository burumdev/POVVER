use crossbeam_channel::{Receiver, Sender, unbounded, bounded };

use crate::{
    simulation::{
        StateAction,
    },
};
use crate::economy::economy_types::EnergyUnit;
use crate::simulation::SimInt;

#[derive(Debug, Clone, PartialEq)]
pub enum MessageEntity {
    Hub,
    PP,
    Factory(SimInt),
}

#[derive(Debug)]
pub enum PPHubSignal {
    BuyFuel(SimInt),
    IncreaseFuelCapacity,
}

#[derive(Debug, PartialEq)]
pub enum HubPPSignal {
    FuelTransfered,
    FuelCapacityIncreased,
}

#[derive(Debug, PartialEq)]
pub struct FactoryEnergyDemand {
    pub factory_id: usize,
    pub energy: EnergyUnit,
}

#[derive(Debug, PartialEq)]
pub enum FactoryHubSignal {
    EnergyDemand(FactoryEnergyDemand)
}

pub struct HubComms {
    pub broadcast_count: usize,
    broadcast_channel: (Sender<StateAction>, Receiver<StateAction>),
    hub_pp_channel: (Sender<HubPPSignal>, Receiver<HubPPSignal>),
    pp_hub_channel: (Sender<PPHubSignal>, Receiver<PPHubSignal>),
    factory_hub_channel: (Sender<FactoryHubSignal>, Receiver<FactoryHubSignal>)
}

impl HubComms {
    pub fn new() -> Self {
        Self {
            broadcast_count: 0,
            broadcast_channel: unbounded(),
            hub_pp_channel: bounded(4),
            pp_hub_channel: bounded(4),
            factory_hub_channel: bounded(128),
        }
    }

}

impl HubComms {
    fn broadcast_sender(&self) -> &Sender<StateAction> {
        &self.broadcast_channel.0
    }

    fn hub_pp_sender(&self) -> &Sender<HubPPSignal> {
        &self.hub_pp_channel.0
    }
}

impl HubComms {
    pub fn clone_broadcast_receiver(&self) -> Receiver<StateAction> {
        self.broadcast_channel.1.clone()
    }

    pub fn clone_pp_hub_sender(&self) -> Sender<PPHubSignal> {
        self.pp_hub_channel.0.clone()
    }

    pub fn clone_factory_hub_sender(&self) -> Sender<FactoryHubSignal> {
        self.factory_hub_channel.0.clone()
    }

    pub fn clone_pp_hub_receiver(&self) -> Receiver<PPHubSignal> {
        self.pp_hub_channel.1.clone()
    }

    pub fn clone_factory_hub_receiver(&self) -> Receiver<FactoryHubSignal> {
        self.factory_hub_channel.1.clone()
    }

    pub fn clone_hub_pp_receiver(&self) -> Receiver<HubPPSignal> {
        self.hub_pp_channel.1.clone()
    }

    pub fn send_state_broadcast(&self, action: StateAction) {
        for _ in 0..self.broadcast_count {
            if let Err(e) = self.broadcast_sender().send(action.clone()) {
                eprintln!("HUB COMMS: Could not send state broadcast signal to one recipient: {e}");
            }
        }
    }

    pub fn hub_to_pp(&self, signal: HubPPSignal) {
        self.hub_pp_sender().send(signal).unwrap();
    }
}
