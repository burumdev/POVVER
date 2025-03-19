use tokio::sync::broadcast as tokio_broadcast;
use slint::SharedString;
use crate::{
    simulation::hub_comms::MessageEntity,
    ui_controller::{LogMessage as UILogMessage, LogLevel as UILogLevel, MessageSource as UIMessageSource},
};

#[derive(Debug, Clone)]
pub enum LogLevel {
    Info,
    Warning,
    Critical,
    Error,
}

impl From<LogLevel> for UILogLevel {
    fn from(ll: LogLevel) -> Self {
        match ll {
            LogLevel::Info => UILogLevel::Info,
            LogLevel::Warning => UILogLevel::Warning,
            LogLevel::Critical => UILogLevel::Critical,
            LogLevel::Error => UILogLevel::Error,
        }
    }
}

impl From<MessageEntity> for UIMessageSource {
    fn from(source: MessageEntity) -> Self {
        match source {
            MessageEntity::Hub => UIMessageSource::Hub,
            MessageEntity::PP => UIMessageSource::PP,
            MessageEntity::Factory(_) => UIMessageSource::Factory,
        }
    }
}

#[derive(Debug, Clone)]
pub struct LogMessage {
    source: MessageEntity,
    message: SharedString,
    log_level: LogLevel,
}
impl From<LogMessage> for UILogMessage {
    fn from(lm: LogMessage) -> Self {
        let mut factory_id = -1;
        if let MessageEntity::Factory(id) = lm.source {
            factory_id = id;
        }

        UILogMessage {
            source: lm.source.into(),
            level: lm.log_level.into(),
            message: lm.message,
            factory_id
        }
    }
}

pub trait Logger {
    fn get_log_prefix(&self) -> String;
    fn get_message_source(&self) -> MessageEntity;
    fn get_log_sender(&self) -> tokio_broadcast::Sender<LogMessage>;
    fn log_console(&self, message: String, level: LogLevel) {
        let level_prefix = match level {
            LogLevel::Info => "",
            LogLevel::Warning => "WARNING: ",
            LogLevel::Critical => "CRITICAL!: ",
            LogLevel::Error => "ERROR!: ",
        };
        println!("{}{}: {}", level_prefix, self.get_log_prefix(), message);
    }

    fn log_ui(&self, message: String, level: LogLevel) {
        let msg = self.get_log_prefix() + ": " + &message;
        self.get_log_sender().send(
            LogMessage {
                source: self.get_message_source(),
                message: SharedString::from(msg),
                log_level: level,
            }
        ).unwrap();
    }

    fn log_ui_console(&self, message: String, level: LogLevel) {
        self.log_ui(message.clone(), level.clone());
        self.log_console(message, level);
    }
}
