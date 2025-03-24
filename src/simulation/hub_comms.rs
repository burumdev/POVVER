use std::{
    any::Any,
    sync::Arc,
    fmt::Debug,
};
use crossbeam_channel::{Sender as CrossbeamSender, Receiver as CrossbeamReceiver, bounded};
use tokio::{
    sync::broadcast as tokio_broadcast
};

use crate::{
    simulation::{
        StateAction,
        SimInt,
        SimFlo,
    },
    economy::economy_types::{EnergyUnit, Money},
};

#[derive(Debug, Clone, PartialEq)]
pub enum MessageEntity {
    Hub,
    PP,
    Factory(SimInt),
}

#[derive(Debug, Clone, PartialEq)]
pub struct FuelReceipt {
    pub amount: SimInt,
    pub price_per_unit: SimFlo,
}

#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub struct PPEnergyOffer {
    pub price_per_unit: Money,
    pub units: EnergyUnit,
    pub to_factory_id: usize,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct FactoryEnergyDemand {
    pub factory_id: usize,
    pub energy_needed: EnergyUnit,
}

#[derive(Debug, PartialEq)]
pub enum HubPPSignal {
    FuelTransfered(FuelReceipt),
    FuelCapacityIncreased,
}

#[derive(Debug, PartialEq)]
pub enum HubFactorySignal {
    EnergyTransfered(EnergyUnit),
}

#[derive(Debug)]
pub enum PPHubSignal {
    BuyFuel(SimInt),
    IncreaseFuelCapacity,
    EnergyToFactory(PPEnergyOffer),
}

#[derive(Debug, PartialEq)]
pub enum FactoryHubSignal {
    EnergyDemand(FactoryEnergyDemand),
}

#[derive(Debug, PartialEq)]
pub enum FactorySignal {
    AcceptPPEnergyOffer(PPEnergyOffer),
    RejectPPEnergyOffer(PPEnergyOffer),
}

pub trait DynSignalable: Send + Sync + Debug {
    fn as_any(&self) -> &dyn Any;
}

macro_rules! impl_dynsignalable {
    ($($structname:ident),+) => {
        $(
            impl DynSignalable for $structname {
                fn as_any(&self) -> &dyn Any {
                    self
                }
            }
        )+
    };
}

impl_dynsignalable!(
    PPHubSignal,
    PPEnergyOffer,
    HubPPSignal,
    HubFactorySignal,
    FactoryEnergyDemand,
    FactorySignal,
    FactoryHubSignal
);

pub type DynamicSignal = Arc<dyn DynSignalable>;
pub type DynamicSender = CrossbeamSender<DynamicSignal>;
pub type DynamicReceiver = CrossbeamReceiver<DynamicSignal>;
pub type DynamicChannel = (CrossbeamSender<DynamicSignal>, CrossbeamReceiver<DynamicSignal>);
pub type BroadcastDynSender = tokio_broadcast::Sender<DynamicSignal>;
pub type BroadcastDynReceiver = tokio_broadcast::Receiver<DynamicSignal>;
pub type BroadcastDynChannel = (BroadcastDynSender, BroadcastDynReceiver);

pub struct HubComms {
    broadcast_state_channel: (tokio_broadcast::Sender<StateAction>, tokio_broadcast::Receiver<StateAction>),
    broadcast_signal_channel: BroadcastDynChannel,
    pp_dyn_channel: DynamicChannel,
    pub from_factory_dyn_channels: Vec<BroadcastDynChannel>,
    pub to_factory_dyn_channels: Vec<DynamicChannel>,
}

impl HubComms {
    pub fn new(factory_count: usize) -> Self {
        let from_factory_dyn_channels = (0..factory_count).map(|_| tokio_broadcast::channel(64)).collect();
        let to_factory_dyn_channels = (0..factory_count).map(|_| bounded(64)).collect();

        Self {
            broadcast_state_channel: tokio_broadcast::channel(64),
            broadcast_signal_channel: tokio_broadcast::channel(64),
            pp_dyn_channel: bounded(64),
            from_factory_dyn_channels,
            to_factory_dyn_channels,
        }
    }
}

impl HubComms {
    fn broadcast_state_sender(&self) -> &tokio_broadcast::Sender<StateAction> {
        &self.broadcast_state_channel.0
    }

    fn broadcast_signal_sender(&self) -> &tokio_broadcast::Sender<DynamicSignal> {
        &self.broadcast_signal_channel.0
    }

    fn pp_dyn_sender(&self) -> &DynamicSender {
        &self.pp_dyn_channel.0
    }

    fn to_factory_dyn_sender(&self, factory_id: usize) -> &CrossbeamSender<DynamicSignal> {
        &self.to_factory_dyn_channels[factory_id].0
    }
}

impl HubComms {
    pub fn clone_broadcast_state_receiver(&self) -> tokio_broadcast::Receiver<StateAction> {
        self.broadcast_state_channel.1.resubscribe()
    }

    pub fn clone_broadcast_signal_receiver(&self) -> tokio_broadcast::Receiver<DynamicSignal> {
        self.broadcast_signal_channel.1.resubscribe()
    }

    pub fn clone_to_factory_dyn_sender(&self, factory_id: usize) -> DynamicSender {
        self.to_factory_dyn_channels[factory_id].0.clone()
    }

    pub fn clone_to_factory_dyn_receiver(&self, factory_id: usize) -> DynamicReceiver {
        self.to_factory_dyn_channels[factory_id].1.clone()
    }

    pub fn clone_from_factory_dyn_sender(&self, factory_id: usize) -> BroadcastDynSender {
        self.from_factory_dyn_channels[factory_id].0.clone()
    }

    pub fn clone_from_factory_dyn_receivers(&self) -> Vec<BroadcastDynReceiver> {
        self.from_factory_dyn_channels.iter().map(|(_, r)| r.resubscribe()).collect()
    }

    pub fn clone_pp_dyn_receiver(&self) -> DynamicReceiver {
        self.pp_dyn_channel.1.clone()
    }

    pub fn clone_pp_dyn_channel(&self) -> DynamicChannel {
        self.pp_dyn_channel.clone()
    }
}

impl HubComms {
    pub fn send_state_broadcast(&self, action: StateAction) {
        if let Err(e) = self.broadcast_state_sender().send(action) {
            eprintln!("HUB COMMS: Could not send state broadcast to one recipient: {e}");
        }
    }

    pub fn send_signal_broadcast(&self, signal: DynamicSignal) {
        if let Err(e) = self.broadcast_signal_sender().send(signal) {
            eprintln!("HUB COMMS: Could not send broadcast signal to one recipient: {e}");
        }
    }

    pub fn hub_to_pp(&self, signal: DynamicSignal) {
        self.pp_dyn_sender().send(signal).unwrap();
    }

    pub fn hub_to_factory(&self, signal: DynamicSignal, factory_id: usize) {
        if let Err(e) = self.to_factory_dyn_sender(factory_id).send(signal) {
            eprintln!("HUB COMMS: Could not send signal to factory No. {factory_id}: {e}");
        }
    }
}