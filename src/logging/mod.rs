use anyhow::Result;
use std::path::PathBuf;
use tracing::Level;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{EnvFilter, Layer, fmt, prelude::*};

/// Enhanced logging configuration
pub struct LoggingConfig {
    pub level: Level,
    pub file_output: bool,
    pub console_output: bool,
    pub log_dir: Option<PathBuf>,
    pub json_format: bool,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: Level::INFO,
            file_output: true,
            console_output: true,
            log_dir: None,
            json_format: false,
        }
    }
}

/// Initialize enhanced logging with file rotation and structured output
///
/// Returns a tuple of (WorkerGuard, log_dir) for optional startup message
pub fn initialize_logging(config: LoggingConfig) -> Result<(Option<WorkerGuard>, Option<PathBuf>)> {
    let mut layers = Vec::new();
    let mut guard = None;

    // Create environment filter
    let env_filter = EnvFilter::new(format!(
        "audio_device_monitor={}",
        config.level.as_str().to_lowercase()
    ));

    // Console output layer
    if config.console_output {
        let console_layer = if config.json_format {
            fmt::layer()
                .json()
                .with_target(true)
                .with_thread_ids(true)
                .with_file(true)
                .with_line_number(true)
                .boxed()
        } else {
            fmt::layer()
                .with_target(true)
                .with_thread_ids(false)
                .with_file(false)
                .with_line_number(false)
                .boxed()
        };
        layers.push(console_layer);
    }

    // File output layer with rotation
    let log_dir = if config.file_output {
        let dir = config.log_dir.clone().unwrap_or_else(|| {
            dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("/tmp"))
                .join(".local/share/audio-device-monitor/logs")
        });

        // Create log directory if it doesn't exist
        std::fs::create_dir_all(&dir)?;

        // Create file appender with daily rotation
        let file_appender = tracing_appender::rolling::daily(&dir, "audio-device-monitor.log");
        let (non_blocking, worker_guard) = tracing_appender::non_blocking(file_appender);
        guard = Some(worker_guard);

        let file_layer = if config.json_format {
            fmt::layer()
                .json()
                .with_target(true)
                .with_thread_ids(true)
                .with_file(true)
                .with_line_number(true)
                .with_writer(non_blocking)
                .boxed()
        } else {
            fmt::layer()
                .with_target(true)
                .with_thread_ids(false)
                .with_file(true)
                .with_line_number(true)
                .with_writer(non_blocking)
                .boxed()
        };
        layers.push(file_layer);

        Some(dir)
    } else {
        None
    };

    // Initialize the subscriber
    tracing_subscriber::registry()
        .with(env_filter)
        .with(layers)
        .init();

    Ok((guard, log_dir))
}

/// Get the default log directory path
pub fn get_default_log_dir() -> Result<PathBuf> {
    let home_dir =
        dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Failed to get home directory"))?;
    Ok(home_dir.join(".local/share/audio-device-monitor/logs"))
}

/// Clean up old log files (keep last N days)
pub fn cleanup_old_logs(log_dir: &PathBuf, keep_days: u64) -> Result<()> {
    use std::time::{Duration, SystemTime};

    let cutoff_time = SystemTime::now() - Duration::from_secs(60 * 60 * 24 * keep_days);

    if !log_dir.exists() {
        return Ok(());
    }

    let entries = std::fs::read_dir(log_dir)?;
    let mut cleaned_count = 0;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() && path.extension().is_some_and(|ext| ext == "log") {
            if let Ok(metadata) = entry.metadata() {
                if let Ok(created) = metadata.created() {
                    if created < cutoff_time {
                        if let Err(e) = std::fs::remove_file(&path) {
                            tracing::warn!(
                                "Failed to remove old log file {}: {}",
                                path.display(),
                                e
                            );
                        } else {
                            cleaned_count += 1;
                            tracing::debug!("Removed old log file: {}", path.display());
                        }
                    }
                }
            }
        }
    }

    if cleaned_count > 0 {
        tracing::info!(
            "Cleaned up {} old log files from {}",
            cleaned_count,
            log_dir.display()
        );
    }

    Ok(())
}
