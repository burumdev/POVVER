use std::{
    any::Any,
    sync::Arc,
    fmt::Debug,
};
use crossbeam_channel::{Receiver, Sender, bounded};
use tokio::{
    sync::broadcast as tokio_broadcast
};

use crate::{
    simulation::{
        StateAction,
        SimInt,
    },
    economy::economy_types::EnergyUnit,
};
use crate::economy::economy_types::Money;
use crate::simulation::SimFlo;

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
    EnergyOfferToFactory(PPEnergyOffer),
}

#[derive(Debug, Clone, PartialEq)]
pub struct FuelReceipt {
    pub amount: SimInt,
    pub price_per_unit: SimFlo,
}

#[derive(Debug, PartialEq)]
pub enum HubPPSignal {
    FuelTransfered(FuelReceipt),
    FuelCapacityIncreased,
}

#[derive(Debug)]
pub struct PPEnergyOffer {
    pub price_per_unit: Money,
    pub units: EnergyUnit,
    pub to_factory_id: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FactoryEnergyDemand {
    pub factory_id: usize,
    pub energy: EnergyUnit,
}

#[derive(Debug, PartialEq)]
pub enum FactoryHubSignal {
    EnergyDemand(FactoryEnergyDemand)
}

pub trait Broadcastable: Send + Sync + Debug {
    fn as_any(&self) -> &dyn Any;
}

macro_rules! impl_broadcastable {
    ($($structname:ident),+) => {
        $(
            impl Broadcastable for $structname {
                fn as_any(&self) -> &dyn Any {
                    self
                }
            }
        )+
    };
}
impl_broadcastable!(FactoryEnergyDemand);

pub struct HubComms {
    broadcast_state_channel: (tokio_broadcast::Sender<StateAction>, tokio_broadcast::Receiver<StateAction>),
    broadcast_signal_channel: (tokio_broadcast::Sender<Arc<dyn Broadcastable>>, tokio_broadcast::Receiver<Arc<dyn Broadcastable>>),
    hub_pp_channel: (Sender<HubPPSignal>, Receiver<HubPPSignal>),
    pp_hub_channel: (Sender<PPHubSignal>, Receiver<PPHubSignal>),
    factory_hub_channel: (Sender<FactoryHubSignal>, Receiver<FactoryHubSignal>)
}

impl HubComms {
    pub fn new() -> Self {
        Self {
            broadcast_state_channel: tokio_broadcast::channel(64),
            broadcast_signal_channel: tokio_broadcast::channel(64),
            hub_pp_channel: bounded(4),
            pp_hub_channel: bounded(4),
            factory_hub_channel: bounded(128),
        }
    }
}

impl HubComms {
    fn broadcast_state_sender(&self) -> &tokio_broadcast::Sender<StateAction> {
        &self.broadcast_state_channel.0
    }

    fn broadcast_signal_sender(&self) -> &tokio_broadcast::Sender<Arc<dyn Broadcastable>> {
        &self.broadcast_signal_channel.0
    }

    fn hub_pp_sender(&self) -> &Sender<HubPPSignal> {
        &self.hub_pp_channel.0
    }
}

impl HubComms {
    pub fn clone_broadcast_state_receiver(&self) -> tokio_broadcast::Receiver<StateAction> {
        self.broadcast_state_channel.1.resubscribe()
    }

    pub fn clone_broadcast_signal_receiver(&self) -> tokio_broadcast::Receiver<Arc<dyn Broadcastable>> {
        self.broadcast_signal_channel.1.resubscribe()
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
        if let Err(e) = self.broadcast_state_sender().send(action) {
            eprintln!("HUB COMMS: Could not send state broadcast to one recipient: {e}");
        }
    }

    pub fn send_signal_broadcast(&self, signal: Arc<dyn Broadcastable>) {
        if let Err(e) = self.broadcast_signal_sender().send(signal) {
            eprintln!("HUB COMMS: Could not send broadcast signal to one recipient: {e}");
        }
    }

    pub fn hub_to_pp(&self, signal: HubPPSignal) {
        self.hub_pp_sender().send(signal).unwrap();
    }
}
