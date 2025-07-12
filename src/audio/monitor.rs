use anyhow::Result;
use tracing::{debug, info};

use super::controller::DeviceController;
use super::listener::CoreAudioListener;
use crate::config::Config;

pub struct AudioDeviceMonitor {
    controller: DeviceController,
    config: Config,
    listener: CoreAudioListener,
}

impl AudioDeviceMonitor {
    pub fn new(config: Config) -> Result<Self> {
        let controller = DeviceController::new()?;
        let listener = CoreAudioListener::new(&config)?;

        info!("Created audio device monitor with CoreAudio listener");

        Ok(Self {
            controller,
            config,
            listener,
        })
    }

    pub async fn start(&self) -> Result<()> {
        info!("Starting audio device monitor");

        // Phase 1: Basic device enumeration
        self.list_initial_devices().await?;

        // Phase 2: Real-time device change monitoring
        info!("Starting real-time device monitoring");

        // This will block and run the CoreAudio event loop
        self.listener.start_monitoring()?;

        Ok(())
    }

    pub async fn start_monitoring_async(&self) -> Result<()> {
        info!("Starting async device monitoring");

        // Show initial devices
        self.list_initial_devices().await?;

        // Register listeners but don't start the run loop yet
        self.listener.register_listeners()?;

        info!("CoreAudio listeners registered, monitoring device changes...");
        println!("Device monitoring active - try plugging/unplugging audio devices");
        println!("Press Ctrl+C to stop");

        Ok(())
    }

    pub fn stop(&self) -> Result<()> {
        info!("Stopping audio device monitor");
        self.listener.stop_monitoring()?;
        Ok(())
    }

    async fn list_initial_devices(&self) -> Result<()> {
        info!("Enumerating initial devices");

        let devices = self.controller.enumerate_devices()?;

        println!("Found {} audio devices:", devices.len());
        for device in &devices {
            println!("  {}", device);
        }

        // Show default devices
        if let Ok(Some(default_input)) = self.controller.get_default_input_device() {
            println!("Default input: {}", default_input.name);
        }

        if let Ok(Some(default_output)) = self.controller.get_default_output_device() {
            println!("Default output: {}", default_output.name);
        }

        Ok(())
    }
}
