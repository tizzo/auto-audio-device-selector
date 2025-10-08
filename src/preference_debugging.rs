//! Preference debugging functionality
//!
//! Provides utilities for checking if current devices match configured preferences
//! and applying preferences when they don't match.

/// Status of current devices compared to configured preferences
#[derive(Debug, PartialEq, Clone)]
pub struct PreferenceStatus {
    /// Whether current output device matches highest priority configured device
    pub output_matches: bool,
    /// Whether current input device matches highest priority configured device  
    pub input_matches: bool,
    /// Name of currently selected output device
    pub current_output: Option<String>,
    /// Name of currently selected input device
    pub current_input: Option<String>,
    /// Name of preferred output device based on configuration
    pub preferred_output: Option<String>,
    /// Name of preferred input device based on configuration
    pub preferred_input: Option<String>,
    /// Alternative name for output device (for API compatibility)
    pub output_device_name: Option<String>,
    /// Alternative name for input device (for API compatibility)
    pub input_device_name: Option<String>,
}

/// Changes made when applying preferences
#[derive(Debug, PartialEq, Clone)]
pub struct PreferenceChanges {
    /// Whether output device was changed
    pub output_changed: bool,
    /// Whether input device was changed
    pub input_changed: bool,
    /// Name of new output device if changed
    pub new_output: Option<String>,
    /// Name of new input device if changed
    pub new_input: Option<String>,
}

impl PreferenceStatus {
    /// Create a new PreferenceStatus with no devices matching
    #[allow(dead_code)]
    pub fn no_matches() -> Self {
        Self {
            output_matches: false,
            input_matches: false,
            current_output: None,
            current_input: None,
            preferred_output: None,
            preferred_input: None,
            output_device_name: None,
            input_device_name: None,
        }
    }

    /// Create a new PreferenceStatus indicating all preferences match
    #[allow(dead_code)]
    pub fn all_match(output_name: String, input_name: String) -> Self {
        Self {
            output_matches: true,
            input_matches: true,
            current_output: Some(output_name.clone()),
            current_input: Some(input_name.clone()),
            preferred_output: Some(output_name.clone()),
            preferred_input: Some(input_name.clone()),
            output_device_name: Some(output_name),
            input_device_name: Some(input_name),
        }
    }
}

impl PreferenceChanges {
    /// Create a new PreferenceChanges with no changes
    pub fn no_changes() -> Self {
        Self {
            output_changed: false,
            input_changed: false,
            new_output: None,
            new_input: None,
        }
    }

    /// Create a new PreferenceChanges indicating both devices changed
    #[allow(dead_code)]
    pub fn both_changed(new_output: String, new_input: String) -> Self {
        Self {
            output_changed: true,
            input_changed: true,
            new_output: Some(new_output),
            new_input: Some(new_input),
        }
    }

    /// Create a new PreferenceChanges indicating only output changed
    #[allow(dead_code)]
    pub fn output_only_changed(new_output: String) -> Self {
        Self {
            output_changed: true,
            input_changed: false,
            new_output: Some(new_output),
            new_input: None,
        }
    }

    /// Create a new PreferenceChanges indicating only input changed
    #[allow(dead_code)]
    pub fn input_only_changed(new_input: String) -> Self {
        Self {
            output_changed: false,
            input_changed: true,
            new_output: None,
            new_input: Some(new_input),
        }
    }
}
