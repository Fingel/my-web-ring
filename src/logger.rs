use std::{
    fs::OpenOptions,
    io::{BufWriter, Write},
    path::PathBuf,
    sync::mpsc::{self, Receiver, Sender},
    thread,
};

use log::{LevelFilter, Metadata, Record, SetLoggerError};

pub struct AsyncFileLogger {
    sender: Sender<Option<String>>,
    level_filter: LevelFilter,
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

    fn log_thread_fn(receiver: Receiver<Option<String>>, path: &PathBuf) {
        let file = match OpenOptions::new().create(true).append(true).open(path) {
            Ok(file) => file,
            Err(err) => panic!("Failed to open log file: {}", err),
        };

        let mut writer = BufWriter::new(file);

        while let Ok(message) = receiver.recv() {
            match message {
                Some(log_message) => {
                    if let Err(e) = writeln!(writer, "{}", log_message) {
                        eprintln!("Failed to write log message: {}", e);
                    }
                    if let Err(e) = writer.flush() {
                        eprintln!("Failed to flush log file: {}", e);
                    }
                }
                None => {
                    // This is the message to shut down the thread.
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
            let message = format!("{} - {}", record.level(), record.args());
            self.sender.send(Some(message)).unwrap();
        }
    }

    fn flush(&self) {}
}

impl Drop for AsyncFileLogger {
    fn drop(&mut self) {
        // Signal the logging thread to shut down
        let _ = self.sender.send(None);
    }
}
