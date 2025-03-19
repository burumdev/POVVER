use std::any::Any;
use std::sync::Arc;
use std::fmt::Debug;
use crossbeam_channel::{Receiver, Sender, unbounded, bounded };

use crate::{
    simulation::{
        StateAction,
        SimInt,
    },
    economy::economy_types::EnergyUnit,
};

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

#[derive(Debug, Clone, PartialEq)]
pub struct FactoryEnergyDemand {
    pub factory_id: usize,
    pub energy: EnergyUnit,
}
impl Broadcastable for FactoryEnergyDemand {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Debug, PartialEq)]
pub enum FactoryHubSignal {
    EnergyDemand(FactoryEnergyDemand)
}

pub trait Broadcastable: Send + Sync + Debug {
    fn as_any(&self) -> &dyn Any;
}

pub struct HubComms {
    pub broadcast_count: usize,
    broadcast_state_channel: (Sender<StateAction>, Receiver<StateAction>),
    broadcast_signal_channel: (Sender<Arc<dyn Broadcastable>>, Receiver<Arc<dyn Broadcastable>>),
    hub_pp_channel: (Sender<HubPPSignal>, Receiver<HubPPSignal>),
    pp_hub_channel: (Sender<PPHubSignal>, Receiver<PPHubSignal>),
    factory_hub_channel: (Sender<FactoryHubSignal>, Receiver<FactoryHubSignal>)
}

impl HubComms {
    pub fn new() -> Self {
        Self {
            broadcast_count: 0,
            broadcast_state_channel: unbounded(),
            broadcast_signal_channel: unbounded(),
            hub_pp_channel: bounded(4),
            pp_hub_channel: bounded(4),
            factory_hub_channel: bounded(128),
        }
    }

}

impl HubComms {
    fn broadcast_state_sender(&self) -> &Sender<StateAction> {
        &self.broadcast_state_channel.0
    }

    fn broadcast_signal_sender(&self) -> &Sender<Arc<dyn Broadcastable>> {
        &self.broadcast_signal_channel.0
    }

    fn hub_pp_sender(&self) -> &Sender<HubPPSignal> {
        &self.hub_pp_channel.0
    }
}

impl HubComms {
    pub fn clone_broadcast_state_receiver(&self) -> Receiver<StateAction> {
        self.broadcast_state_channel.1.clone()
    }

    pub fn clone_broadcast_signal_receiver(&self) -> Receiver<Arc<dyn Broadcastable>> {
        self.broadcast_signal_channel.1.clone()
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
            if let Err(e) = self.broadcast_state_sender().send(action.clone()) {
                eprintln!("HUB COMMS: Could not send state broadcast signal to one recipient: {e}");
            }
        }
    }

    pub fn send_signal_broadcast(&self, signal: Arc<dyn Broadcastable>) {
        for _ in 0..self.broadcast_count {
            if let Err(e) = self.broadcast_signal_sender().send(signal.clone()) {
                eprintln!("HUB COMMS: Could not send signal signal to one recipient: {e}");
            }
        }
    }

    pub fn hub_to_pp(&self, signal: HubPPSignal) {
        self.hub_pp_sender().send(signal).unwrap();
    }
}
