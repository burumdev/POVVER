use crossbeam_channel::{Receiver, Sender, unbounded, bounded, TryRecvError };

use crate::{
    simulation::StateAction,
};
use crate::simulation::hub_types::{HubPPSignal, PPHubSignal};

pub struct HubComms {
    pub broadcast_count: usize,
    broadcast_channel: (Sender<StateAction>, Receiver<StateAction>),
    hub_pp_channel: (Sender<HubPPSignal>, Receiver<HubPPSignal>),
    pp_hub_channel: (Sender<PPHubSignal>, Receiver<PPHubSignal>),
}

impl HubComms {
    pub fn new() -> Self {
        Self {
            broadcast_count: 0,
            broadcast_channel: unbounded(),
            hub_pp_channel: bounded(4),
            pp_hub_channel: bounded(4),
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

    fn pp_hub_receiver(&self) -> &Receiver<PPHubSignal> {
        &self.pp_hub_channel.1
    }
}

impl HubComms {
    pub fn clone_broadcast_sender(&self) -> Sender<StateAction> {
        self.broadcast_channel.0.clone()
    }

    pub fn clone_broadcast_receiver(&self) -> Receiver<StateAction> {
        self.broadcast_channel.1.clone()
    }

    pub fn clone_pp_hub_sender(&self) -> Sender<PPHubSignal> {
        self.pp_hub_channel.0.clone()
    }

    pub fn clone_pp_hub_receiver(&self) -> Receiver<PPHubSignal> {
        self.pp_hub_channel.1.clone()
    }

    pub fn clone_hub_pp_sender(&self) -> Sender<HubPPSignal> {
        self.hub_pp_channel.0.clone()
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
