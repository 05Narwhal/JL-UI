#[cfg(feature = "debug")]
pub mod init {
    use chrono::Local;
    use colored::Colorize;
    use env_logger::Env;
    use log::{info, Level};
    use std::{
        fs,
        io::{BufRead, BufReader, Write},
        path::{Path, PathBuf},
    };

    use crate::modules::global::traits::PathTrait;

    #[derive(Debug, Clone)]
    pub struct Logger {
        pub logs_path: String,
        pub current_log_path: String,
    }

    impl Logger {
        /// Initializes the logger with file logging and rotation
        pub fn initialize(logs_path: &str, level: log::Level) -> Self {
            // Ensure the logs directory exists
            fs::create_dir_all(logs_path).expect("Failed to create logs directory");

            // Generate the current log file path with a timestamp
            let current_log_path = format!(
                "{}/{}.log",
                logs_path,
                Local::now().format("%Y-%m-%d_%H-%M-%S")
            );

            // Set up the logger
            let current_log_path_clone = current_log_path.clone();
            env_logger::Builder::from_env(Env::default().default_filter_or(log::Level::Trace.to_string()))
            .format(move |_buf, record| {
                let level_colored = match record.level() {
                    Level::Error => record.level().to_string().red(),
                    Level::Warn => record.level().to_string().yellow(),
                    Level::Info => record.level().to_string().green(),
                    Level::Debug => record.level().to_string().blue(),
                    Level::Trace => record.level().to_string().purple(),
                };

                let _file_shortened = if record.file().is_some() {
                    Path::new(record.file().unwrap()).display_relative()
                } else {
                    "unknown".to_string()
                };

                let log_message = format!(
                    "[{}] [{}] - {}",
                    Local::now().format("%Y-%m-%d / %H:%M:%S").to_string().blue(),
                    level_colored,
                    // format!("{:?}:{:?}", file_shortened, record.line().unwrap_or(0)).cyan(),
                    record.args()
                );

                let log_msg_no_color = format!(
                    "[{}] [{}] [{}] - {}",
                    Local::now().format("%Y-%m-%d / %H:%M:%S").to_string(),
                    record.level(),
                    format!("{:?}:{:?}", record.file().unwrap_or("unknown"), record.line().unwrap_or(0)),
                    record.args()
                );

                // Write to the console if the log level is above the specified level
                if record.level() <= level {
                    println!("{}", log_message);
                }

                // Append the log message to the log file
                let mut file = fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&current_log_path_clone)
                    .expect("Failed to open log file");
                writeln!(file, "{}", log_msg_no_color)?;

                Ok(())
            })
            .init();

            info!("Logger initialized at path: \"{}\"", Path::new(&logs_path).display_relative());

            // Perform log file rotation
            Self::rotate_logs(logs_path, 10);

            Self {
                logs_path: logs_path.to_string(),
                current_log_path,
            }
        }

        /// Rotates logs, keeping only the `max_files` most recent logs
        fn rotate_logs(logs_path: &str, max_files: usize) {
            let mut log_files: Vec<PathBuf> = fs::read_dir(logs_path)
            .expect("Failed to read logs directory")
            .filter_map(|entry| entry.ok().map(|e| e.path()))
            .filter(|path| path.is_file() && path.extension().map_or(false, |ext| ext == "log"))
            .collect();

            // Sort log files by modification time (newest first)
            log_files.sort_by(|a, b| {
                b.metadata()
                    .and_then(|m| m.modified())
                    .unwrap_or_else(|_| std::time::SystemTime::UNIX_EPOCH)
                    .cmp(&a.metadata().and_then(|m| m.modified()).unwrap_or_else(|_| std::time::SystemTime::UNIX_EPOCH))
                });

                // Remove older log files, keeping only the `max_files` most recent ones
                if log_files.len() > max_files {
                for file in log_files.iter().skip(max_files) {
                    fs::remove_file(file).expect("Failed to remove old log file");
                }
            }
        }

        pub fn read_last_line(&self) -> Option<String> {
            let file = fs::File::open(&self.current_log_path).ok()?;
            let reader = BufReader::new(file);
            reader
                .lines()
                .filter_map(|line| line.ok())
                .last() // Get the last line
        }
    }
}
