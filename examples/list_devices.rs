use anyhow::Result;
use audio_device_monitor::audio::DeviceController;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize simple console logging
    tracing_subscriber::fmt::init();
    
    println!("Audio Device Enumeration Example");
    println!("=================================");
    
    let controller = DeviceController::new()?;
    
    // List all devices
    println!("\n--- All Audio Devices ---");
    let devices = controller.enumerate_devices()?;
    
    if devices.is_empty() {
        println!("No audio devices found!");
        return Ok(());
    }
    
    for (i, device) in devices.iter().enumerate() {
        println!("{}. {}", i + 1, device);
    }
    
    // Show default devices
    println!("\n--- Default Devices ---");
    
    if let Ok(Some(default_input)) = controller.get_default_input_device() {
        println!("Default Input:  {}", default_input);
    } else {
        println!("Default Input:  None");
    }
    
    if let Ok(Some(default_output)) = controller.get_default_output_device() {
        println!("Default Output: {}", default_output);
    } else {
        println!("Default Output: None");
    }
    
    println!("\n--- Device Details ---");
    for device in &devices {
        if let Ok(info) = controller.get_device_info(device) {
            println!("Device: {}", info.name);
            println!("  UID: {}", info.uid);
            println!("  Type: {}", info.device_type);
            println!("  Default: {}", info.is_default);
            println!();
        }
    }
    
    Ok(())
}