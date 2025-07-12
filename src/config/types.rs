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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationConfig {
    pub show_device_changes: bool,
    pub show_switching_actions: bool,
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
            show_device_changes: true,
            show_switching_actions: true,
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

        let config: Config = toml::from_str(&config_content)
            .with_context(|| format!("Failed to parse configuration file: {}", path.display()))?;

        info!("Configuration loaded successfully");
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

        // Create parent directories
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!("Failed to create config directory: {}", parent.display())
            })?;
        }

        config.save(path.to_str())?;

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
