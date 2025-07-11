use anyhow::Result;
use tracing::{debug, info};

use crate::config::Config;
use super::controller::DeviceController;

pub struct AudioDeviceMonitor {
    controller: DeviceController,
    config: Config,
}

impl AudioDeviceMonitor {
    pub fn new(config: Config) -> Result<Self> {
        let controller = DeviceController::new()?;
        
        info!("Created audio device monitor");
        
        Ok(Self {
            controller,
            config,
        })
    }
    
    pub async fn start(&self) -> Result<()> {
        info!("Starting audio device monitor");
        
        // Phase 1: Basic device enumeration
        self.list_initial_devices().await?;
        
        // Phase 2: Device change monitoring (to be implemented)
        println!("Device monitoring will be implemented in Phase 2");
        
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
    
    // This will be expanded in Phase 2 with CoreAudio property listeners
    pub fn register_device_change_listeners(&self) -> Result<()> {
        debug!("Registering device change listeners (Phase 2)");
        
        // CoreAudio property listeners will be implemented here
        // - AudioObjectAddPropertyListener for device list changes
        // - AudioObjectAddPropertyListener for default device changes
        
        Ok(())
    }
}