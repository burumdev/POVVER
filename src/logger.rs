use tokio::sync::broadcast as tokio_broadcast;
use slint::{SharedString, ToSharedString};
use crate::simulation::SimInt;

#[derive(Debug, Clone)]
pub enum MessageSource {
    Hub,
    PP,
    Factory(SimInt),
}

#[derive(Debug, Clone)]
pub struct LogMessage {
    source: MessageSource,
    message: SharedString,
}

pub trait Logger {
    fn get_log_prefix(&self) -> String;
    fn get_message_source(&self) -> MessageSource;
    fn get_log_sender(&self) -> tokio_broadcast::Sender<LogMessage>;
    fn log_console(&self, message: String) {
        println!("{}: {}", self.get_log_prefix(), message);
    }

    fn log_ui(&self, message: String) {
        let msg = self.get_log_prefix() + ": " + &message;
        self.get_log_sender().send(
            LogMessage {
                source: self.get_message_source(),
                message: SharedString::from(msg),
            }
        ).unwrap();
    }

    fn log_ui_console(&self, message: String) {
        self.log_ui(message.clone());
        self.log_console(message);
    }
}
