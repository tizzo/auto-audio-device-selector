use anyhow::Result;
use signal_hook::consts::signal::*;
use signal_hook_tokio::Signals;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::mpsc;
use tokio_stream::StreamExt;
use tracing::{info, warn};

/// Signal types that can be received
#[derive(Debug, Clone, Copy)]
pub enum SignalType {
    // Used by signal handlers for graceful service shutdown (SIGTERM/SIGINT)
    #[allow(dead_code)]
    Shutdown,
    // Used by signal handlers for configuration reload (SIGHUP)
    #[allow(dead_code)]
    Reload,
}

/// Handles system signals for graceful shutdown and configuration reload
#[derive(Clone)]
pub struct SignalHandler {
    shutdown_flag: Arc<AtomicBool>,
    // Used by tokio-based signal handling system for async signal communication
    #[allow(dead_code)]
    signal_sender: Option<mpsc::UnboundedSender<SignalType>>,
}

impl SignalHandler {
    pub fn new() -> Self {
        Self {
            shutdown_flag: Arc::new(AtomicBool::new(false)),
            signal_sender: None,
        }
    }

    // Called by service managers that need async signal communication via channels
    #[allow(dead_code)]
    pub fn with_sender(signal_sender: mpsc::UnboundedSender<SignalType>) -> Self {
        Self {
            shutdown_flag: Arc::new(AtomicBool::new(false)),
            signal_sender: Some(signal_sender),
        }
    }

    /// Get a reference to the shutdown flag
    // Called by service main loops to check for shutdown requests via atomic flag
    #[allow(dead_code)]
    pub fn shutdown_flag(&self) -> Arc<AtomicBool> {
        self.shutdown_flag.clone()
    }

    /// Start listening for shutdown signals
    // Called by tokio-based service managers for async signal handling with SIGTERM/SIGINT/SIGHUP
    #[allow(dead_code)]
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

                    // Send shutdown signal if sender is available
                    if let Some(sender) = &self.signal_sender {
                        let _ = sender.send(SignalType::Shutdown);
                    }
                    break;
                }
                SIGHUP => {
                    info!("Received SIGHUP signal, reloading configuration");

                    // Send reload signal if sender is available
                    if let Some(sender) = &self.signal_sender {
                        if let Err(e) = sender.send(SignalType::Reload) {
                            warn!("Failed to send reload signal: {}", e);
                        } else {
                            info!("Configuration reload signal sent");
                        }
                    } else {
                        warn!("No signal receiver configured, reload request ignored");
                    }
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
