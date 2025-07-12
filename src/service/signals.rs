use anyhow::Result;
use signal_hook::consts::signal::*;
use signal_hook_tokio::Signals;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio_stream::StreamExt;
use tracing::{info, warn};

/// Handles system signals for graceful shutdown
#[derive(Clone)]
pub struct SignalHandler {
    shutdown_flag: Arc<AtomicBool>,
}

impl SignalHandler {
    pub fn new() -> Self {
        Self {
            shutdown_flag: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Get a reference to the shutdown flag
    pub fn shutdown_flag(&self) -> Arc<AtomicBool> {
        self.shutdown_flag.clone()
    }

    /// Start listening for shutdown signals
    pub async fn listen_for_signals(&self) -> Result<()> {
        let mut signals = Signals::new([SIGTERM, SIGINT, SIGHUP])?;

        info!("Signal handler initialized, listening for SIGTERM, SIGINT, SIGHUP");

        while let Some(signal) = signals.next().await {
            match signal {
                SIGTERM | SIGINT => {
                    info!(
                        "Received shutdown signal ({}), initiating graceful shutdown",
                        signal
                    );
                    self.shutdown_flag.store(true, Ordering::Relaxed);
                    break;
                }
                SIGHUP => {
                    info!("Received SIGHUP signal, reloading configuration");
                    // TODO: Implement configuration reload
                    warn!("Configuration reload not yet implemented");
                }
                _ => {
                    warn!("Received unexpected signal: {}", signal);
                }
            }
        }

        Ok(())
    }

    /// Check if shutdown has been requested
    #[allow(dead_code)]
    pub fn is_shutdown_requested(&self) -> bool {
        self.shutdown_flag.load(Ordering::Relaxed)
    }
}

impl Default for SignalHandler {
    fn default() -> Self {
        Self::new()
    }
}
