use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub general: GeneralConfig,

    #[serde(default)]
    pub notifications: NotificationConfig,

    #[serde(default)]
    pub output_devices: Vec<DeviceRule>,

    #[serde(default)]
    pub input_devices: Vec<DeviceRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    pub check_interval_ms: u64,
    pub log_level: String,
    pub daemon_mode: bool,
}

// Helper struct for deserialization that preserves field presence information
#[derive(Debug, Clone, Deserialize)]
struct NotificationConfigHelper {
    #[serde(default)]
    show_device_availability: Option<bool>, // None = not present, Some(x) = explicitly set
    #[serde(default = "default_show_switching_actions")]
    show_switching_actions: bool,
    #[serde(alias = "show_device_changes")]
    show_device_changes: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(from = "NotificationConfigHelper")]
pub struct NotificationConfig {
    pub show_device_availability: bool, // Device connect/disconnect notifications
    pub show_switching_actions: bool,   // Device switching notifications

    // Keep old field for backward compatibility
    #[serde(skip)]
    pub show_device_changes: Option<bool>,
}

fn default_show_switching_actions() -> bool {
    true
}

impl From<NotificationConfigHelper> for NotificationConfig {
    fn from(helper: NotificationConfigHelper) -> Self {
        let was_explicitly_set = helper.show_device_availability.is_some();
        let mut result = NotificationConfig {
            show_device_availability: helper.show_device_availability.unwrap_or(false),
            show_switching_actions: helper.show_switching_actions,
            show_device_changes: helper.show_device_changes,
        };

        // Apply migration logic with presence information
        result = result.migrate_with_presence_info(was_explicitly_set);
        result
    }
}

impl NotificationConfig {
    /// Handle backward compatibility for old config files
    /// This method is primarily for external callers who need to migrate configs manually
    pub fn migrate_from_old_config(mut self) -> Self {
        // For external callers, we don't have presence information
        // so we use the conservative approach: only migrate when old field exists
        // and new field is false (likely a migration scenario)
        if let Some(old_value) = self.show_device_changes {
            if !self.show_device_availability && old_value {
                self.show_device_availability = old_value;
            }
        }
        self.show_device_changes = None;
        self
    }

    /// Internal method used during deserialization to handle migration
    fn migrate_with_presence_info(mut self, was_explicitly_set: bool) -> Self {
        if let Some(old_value) = self.show_device_changes {
            // Only migrate if the new field was NOT explicitly set in the TOML
            if !was_explicitly_set {
                self.show_device_availability = old_value;
            }
            // If the field was explicitly set, respect that value and don't migrate
        }
        self.show_device_changes = None;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceRule {
    pub name: String,
    pub weight: u32,
    pub match_type: MatchType,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MatchType {
    Exact,
    Contains,
    StartsWith,
    EndsWith,
    Regex,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            check_interval_ms: 1000,
            log_level: "info".to_string(),
            daemon_mode: false,
        }
    }
}

impl Default for NotificationConfig {
    fn default() -> Self {
        Self {
            show_device_availability: false, // Default: no device availability notifications
            show_switching_actions: true,    // Default: show switching notifications
            show_device_changes: None,       // Backward compatibility field
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
            notifications: NotificationConfig::default(),
            output_devices: vec![
                DeviceRule {
                    name: "AirPods".to_string(),
                    weight: 100,
                    match_type: MatchType::Contains,
                    enabled: true,
                },
                DeviceRule {
                    name: "MacBook Pro Speakers".to_string(),
                    weight: 10,
                    match_type: MatchType::Exact,
                    enabled: true,
                },
            ],
            input_devices: vec![
                DeviceRule {
                    name: "AirPods".to_string(),
                    weight: 100,
                    match_type: MatchType::Contains,
                    enabled: true,
                },
                DeviceRule {
                    name: "MacBook Pro Microphone".to_string(),
                    weight: 10,
                    match_type: MatchType::Exact,
                    enabled: true,
                },
            ],
        }
    }
}

impl Config {
    pub fn load(config_path: Option<&str>) -> Result<Self> {
        let path = match config_path {
            Some(path) => PathBuf::from(path),
            None => Self::default_config_path()?,
        };

        debug!("Loading configuration from: {}", path.display());

        if !path.exists() {
            info!("Configuration file not found, creating default configuration");
            return Self::create_default_config(&path);
        }

        let config_content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read configuration file: {}", path.display()))?;

        let mut config: Config = toml::from_str(&config_content)
            .with_context(|| format!("Failed to parse configuration file: {}", path.display()))?;

        // Handle backward compatibility for notification config
        config.notifications = config.notifications.migrate_from_old_config();

        debug!("Configuration loaded successfully");
        Ok(config)
    }

    pub fn save(&self, config_path: Option<&str>) -> Result<()> {
        let path = match config_path {
            Some(path) => PathBuf::from(path),
            None => Self::default_config_path()?,
        };

        // Create parent directories if they don't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!("Failed to create config directory: {}", parent.display())
            })?;
        }

        let config_content =
            toml::to_string_pretty(self).context("Failed to serialize configuration")?;

        fs::write(&path, config_content)
            .with_context(|| format!("Failed to write configuration file: {}", path.display()))?;

        info!("Configuration saved to: {}", path.display());
        Ok(())
    }

    fn default_config_path() -> Result<PathBuf> {
        let home_dir = dirs::home_dir().context("Failed to get home directory")?;

        Ok(home_dir.join(".config/audio-device-monitor/config.toml"))
    }

    fn create_default_config(path: &Path) -> Result<Self> {
        let config = Config::default();

        // Try to create parent directories, but don't fail if we can't
        // This handles cases where the path is invalid or we don't have permissions
        if let Some(parent) = path.parent() {
            if let Err(e) = fs::create_dir_all(parent) {
                warn!(
                    "Could not create config directory {}: {}. Using default config without saving.",
                    parent.display(),
                    e
                );
                return Ok(config);
            }
        }

        // Try to save the config, but don't fail if we can't
        if let Err(e) = config.save(path.to_str()) {
            warn!(
                "Could not save default config to {}: {}. Using default config.",
                path.display(),
                e
            );
            return Ok(config);
        }

        info!("Created default configuration file: {}", path.display());
        Ok(config)
    }
}

impl DeviceRule {
    pub fn matches(&self, device_name: &str) -> bool {
        if !self.enabled {
            return false;
        }

        match self.match_type {
            MatchType::Exact => device_name == self.name,
            MatchType::Contains => device_name.contains(&self.name),
            MatchType::StartsWith => device_name.starts_with(&self.name),
            MatchType::EndsWith => device_name.ends_with(&self.name),
            MatchType::Regex => {
                // For now, treat regex as contains. Will implement proper regex later
                warn!("Regex matching not yet implemented, using contains instead");
                device_name.contains(&self.name)
            }
        }
    }
}
