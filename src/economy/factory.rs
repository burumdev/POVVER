use std::thread;
use crate::{
    app_state::FactoryStateData,
    utils_data::ReadOnlyRwLock,
};

pub struct Factory {
    state_ro: ReadOnlyRwLock<FactoryStateData>,
}

impl Factory {
    pub fn new(state_ro: ReadOnlyRwLock<FactoryStateData>) -> Self {
        Self {
            state_ro,
        }
    }
}

impl Factory {
    pub fn start(&self) -> thread::JoinHandle<()> {
        thread::spawn(move || {

        })
    }
}