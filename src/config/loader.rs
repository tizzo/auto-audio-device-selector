use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};

use crate::system::FileSystemInterface;

use super::types::Config;

/// Configuration loader that uses dependency injection for file system operations
pub struct ConfigLoader<F: FileSystemInterface> {
    file_system: F,
    config_path: PathBuf,
}

impl<F: FileSystemInterface> ConfigLoader<F> {
    pub fn new(file_system: F, config_path: PathBuf) -> Self {
        Self {
            file_system,
            config_path,
        }
    }

    /// Load configuration from the configured path
    pub fn load_config(&self) -> Result<Config> {
        debug!("Loading configuration from: {}", self.config_path.display());

        if !self.file_system.config_file_exists(&self.config_path) {
            info!("Configuration file not found, creating default configuration");
            return self.create_default_config();
        }

        let config_content = self
            .file_system
            .read_config_file(&self.config_path)
            .with_context(|| {
                format!(
                    "Failed to read configuration file: {}",
                    self.config_path.display()
                )
            })?;

        let mut config: Config = toml::from_str(&config_content).with_context(|| {
            format!(
                "Failed to parse configuration file: {}",
                self.config_path.display()
            )
        })?;

        // Handle backward compatibility for notification config
        config.notifications = config.notifications.migrate_from_old_config();

        debug!("Configuration loaded successfully");
        Ok(config)
    }

    /// Save configuration to the configured path
    pub fn save_config(&self, config: &Config) -> Result<()> {
        debug!("Saving configuration to: {}", self.config_path.display());

        // Create parent directories if they don't exist
        if let Some(parent) = self.config_path.parent() {
            self.file_system
                .create_config_dir(parent)
                .with_context(|| {
                    format!("Failed to create config directory: {}", parent.display())
                })?;
        }

        let config_content =
            toml::to_string_pretty(config).context("Failed to serialize configuration")?;

        self.file_system
            .write_config_file(&self.config_path, &config_content)
            .with_context(|| {
                format!(
                    "Failed to write configuration file: {}",
                    self.config_path.display()
                )
            })?;

        info!("Configuration saved to: {}", self.config_path.display());
        Ok(())
    }

    /// Reload configuration from file (useful for config hot reloading)
    // Called at runtime by service_v2 when SIGHUP signal is received for configuration hot-reload
    #[allow(dead_code)]
    pub fn reload_config(&self) -> Result<Config> {
        debug!("Reloading configuration");
        self.load_config()
    }

    /// Check if configuration file has been modified since last load
    pub fn is_config_modified(&self, last_modified: std::time::SystemTime) -> Result<bool> {
        if !self.file_system.config_file_exists(&self.config_path) {
            return Ok(false);
        }

        let current_modified = self
            .file_system
            .get_config_modified_time(&self.config_path)?;
        Ok(current_modified > last_modified)
    }

    /// Get the configuration file path
    pub fn get_config_path(&self) -> &Path {
        &self.config_path
    }

    /// Check if the configuration file exists
    // Called at runtime by CLI commands and service initialization to validate config presence
    #[allow(dead_code)]
    pub fn config_exists(&self) -> bool {
        self.file_system.config_file_exists(&self.config_path)
    }

    /// Create and save a default configuration
    fn create_default_config(&self) -> Result<Config> {
        let config = Config::default();

        // Try to create parent directories, but don't fail if we can't
        if let Some(parent) = self.config_path.parent() {
            if let Err(e) = self.file_system.create_config_dir(parent) {
                warn!(
                    "Could not create config directory {}: {}. Using default config without saving.",
                    parent.display(),
                    e
                );
                return Ok(config);
            }
        }

        // Try to save the config, but don't fail if we can't
        if let Err(e) = self.save_config(&config) {
            warn!(
                "Could not save default config to {}: {}. Using default config.",
                self.config_path.display(),
                e
            );
            return Ok(config);
        }

        info!(
            "Created default configuration file: {}",
            self.config_path.display()
        );
        Ok(config)
    }

    /// Get reference to the file system (for testing)
    // Called by test code to access mock file system for verification
    #[cfg(any(test, feature = "test-mocks"))]
    #[allow(dead_code)]
    pub fn get_file_system(&self) -> &F {
        &self.file_system
    }
}

// Convenience constructor for production use with StandardFileSystem
impl ConfigLoader<crate::system::StandardFileSystem> {
    // Called at runtime by production code that needs to create config loader with real file system
    #[allow(dead_code)]
    pub fn new_production(config_path: PathBuf) -> Self {
        Self::new(crate::system::StandardFileSystem, config_path)
    }

    /// Create a production config loader with the default path
    // Called at runtime by service and CLI initialization when no custom config path is provided
    #[allow(dead_code)]
    pub fn new_with_default_path() -> Result<Self> {
        let config_path = Self::default_config_path()?;
        Ok(Self::new_production(config_path))
    }

    /// Get the default configuration path
    pub fn default_config_path() -> Result<PathBuf> {
        let home_dir = dirs::home_dir().context("Failed to get home directory")?;
        Ok(home_dir.join(".config/audio-device-monitor/config.toml"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::system::MockFileSystem;
    use std::path::PathBuf;

    #[test]
    fn test_load_nonexistent_config_creates_default() {
        let mock_fs = MockFileSystem::new();
        let config_path = PathBuf::from("/test/config.toml");
        let loader = ConfigLoader::new(mock_fs, config_path.clone());

        let config = loader.load_config().unwrap();

        // Should be default config
        assert_eq!(config.general.check_interval_ms, 1000);
        assert!(!config.notifications.show_device_availability);
        assert!(config.notifications.show_switching_actions);
    }

    #[test]
    fn test_load_existing_config() {
        let mock_fs = MockFileSystem::new();
        let config_path = PathBuf::from("/test/config.toml");

        // Add a config file to the mock filesystem
        let config_content = r#"
[general]
check_interval_ms = 2000
log_level = "debug"
daemon_mode = true

[notifications]
show_device_availability = true
show_switching_actions = false
"#;
        mock_fs.add_file(&config_path, config_content.to_string());

        let loader = ConfigLoader::new(mock_fs, config_path);
        let config = loader.load_config().unwrap();

        assert_eq!(config.general.check_interval_ms, 2000);
        assert_eq!(config.general.log_level, "debug");
        assert!(config.general.daemon_mode);
        assert!(config.notifications.show_device_availability);
        assert!(!config.notifications.show_switching_actions);
    }

    #[test]
    fn test_save_config() {
        let mock_fs = MockFileSystem::new();
        let config_path = PathBuf::from("/test/config.toml");
        let loader = ConfigLoader::new(mock_fs.clone(), config_path.clone());

        let config = Config::default();
        loader.save_config(&config).unwrap();

        // Verify the file was written
        let write_calls = mock_fs.get_write_calls();
        assert_eq!(write_calls.len(), 1);
        assert_eq!(write_calls[0].0, config_path);

        // Verify directory creation was called
        let dir_calls = mock_fs.get_directory_creation_calls();
        assert_eq!(dir_calls.len(), 1);
        assert_eq!(dir_calls[0], PathBuf::from("/test"));
    }

    #[test]
    fn test_reload_config() {
        let mock_fs = MockFileSystem::new();
        let config_path = PathBuf::from("/test/config.toml");

        let config_content = r#"[general]
check_interval_ms = 3000
log_level = "debug"
daemon_mode = false

[notifications]
show_device_availability = false
show_switching_actions = true
"#;
        mock_fs.add_file(&config_path, config_content.to_string());

        let loader = ConfigLoader::new(mock_fs, config_path);
        let config = loader.reload_config().unwrap();

        assert_eq!(config.general.check_interval_ms, 3000);
        assert_eq!(config.general.log_level, "debug");
    }

    #[test]
    fn test_config_exists() {
        let mock_fs = MockFileSystem::new();
        let config_path = PathBuf::from("/test/config.toml");
        let loader = ConfigLoader::new(mock_fs.clone(), config_path.clone());

        assert!(!loader.config_exists());

        mock_fs.add_file(&config_path, "test content".to_string());
        assert!(loader.config_exists());
    }
}
