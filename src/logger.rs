use tokio::sync::broadcast as tokio_broadcast;
use slint::{SharedString, ToSharedString};

pub trait Logger {
    fn get_log_prefix(&self) -> String;
    fn get_log_sender(&self) -> tokio_broadcast::Sender<SharedString>;
    fn log_console(&self, message: String) {
        println!("{}: {}", self.get_log_prefix(), message);
    }

    fn log_ui(&self, message: String) {
        let msg = self.get_log_prefix() + ": " + &message;
        self.get_log_sender().send(msg.to_shared_string()).unwrap();
    }

    fn log_ui_console(&self, message: String) {
        self.log_ui(message.clone());
        self.log_console(message);
    }
}
