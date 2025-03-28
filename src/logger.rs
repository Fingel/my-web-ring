use chrono::Local;
use log::{LevelFilter, Metadata, Record, SetLoggerError};
use std::{
    fs::OpenOptions,
    io::{BufWriter, Write},
    path::PathBuf,
    sync::mpsc::{self, Receiver, Sender},
    thread,
};

pub struct AsyncFileLogger {
    sender: Sender<AsyncFileLoggerMessage>,
    level_filter: LevelFilter,
}

enum AsyncFileLoggerMessage {
    Log(String),
    Shutdown,
}

impl AsyncFileLogger {
    pub fn init(path: PathBuf, level_filter: LevelFilter) -> Result<(), SetLoggerError> {
        let (sender, receiver) = mpsc::channel();

        thread::spawn(move || {
            Self::log_thread_fn(receiver, &path);
        });

        let logger = Self {
            sender,
            level_filter,
        };

        log::set_max_level(level_filter);
        log::set_boxed_logger(Box::new(logger))
    }

    fn log_thread_fn(receiver: Receiver<AsyncFileLoggerMessage>, path: &PathBuf) {
        let file = match OpenOptions::new().create(true).append(true).open(path) {
            Ok(file) => file,
            Err(err) => panic!("Failed to open log file: {}", err),
        };

        let mut writer = BufWriter::new(file);

        while let Ok(message) = receiver.recv() {
            match message {
                AsyncFileLoggerMessage::Log(log_message) => {
                    if let Err(e) = writeln!(writer, "{}", log_message) {
                        eprintln!("Failed to write log message: {}", e);
                    }
                    if let Err(e) = writer.flush() {
                        eprintln!("Failed to flush log file: {}", e);
                    }
                }
                AsyncFileLoggerMessage::Shutdown => {
                    break;
                }
            }
        }
    }
}

impl log::Log for AsyncFileLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.level_filter
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
            let message = format!("{} - {} - {}", record.level(), now, record.args(),);
            self.sender
                .send(AsyncFileLoggerMessage::Log(message))
                .unwrap();
        }
    }

    fn flush(&self) {}
}

impl Drop for AsyncFileLogger {
    fn drop(&mut self) {
        let _ = self.sender.send(AsyncFileLoggerMessage::Shutdown);
    }
}
