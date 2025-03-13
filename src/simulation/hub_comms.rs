use crossbeam_channel::{Receiver, Sender, unbounded };

use crate::{
    simulation::StateAction,
};

pub struct HubComms {
    broadcast_channel: (Sender<StateAction>, Receiver<StateAction>),
    pub broadcast_count: usize,
}

impl HubComms {
    pub fn new() -> Self {
        Self {
            broadcast_channel: unbounded(),
            broadcast_count: 0,
        }
    }

}

impl HubComms {
    fn broadcast_sender(&self) -> &Sender<StateAction> {
        &self.broadcast_channel.0
    }

}

impl HubComms {
    pub fn broadcast_receiver(&self) -> Receiver<StateAction> {
        self.broadcast_channel.1.clone()
    }

    pub fn send_state_broadcast(&self, action: StateAction) {
        for _ in 0..self.broadcast_count {
            if let Err(e) = self.broadcast_sender().send(action.clone()) {
                eprintln!("HUB COMMS: Could not send state broadcast signal to one recipient: {e}");
            }
        }
    }
}