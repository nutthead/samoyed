//! Configuration structures for samoyed.toml
//!
//! Defines the TOML schema and default configurations for different project types.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::project::ProjectType;

/// Main configuration structure for samoyed.toml
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SamoyedConfig {
    /// Hook definitions (required)
    pub hooks: HashMap<String, String>,

    /// Optional settings
    #[serde(default, skip_serializing_if = "SamoyedSettings::is_default")]
    pub settings: SamoyedSettings,
}

/// Optional settings section for samoyed.toml
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct SamoyedSettings {
    /// Directory for hook files (default: ".samoyed")
    #[serde(default = "default_hook_directory")]
    pub hook_directory: String,

    /// Enable debug output during hook execution (default: false)
    #[serde(default)]
    pub debug: bool,

    /// Stop on first hook failure vs continue (default: true)
    #[serde(default = "default_fail_fast")]
    pub fail_fast: bool,

    /// Skip all hooks if true (default: false)
    #[serde(default)]
    pub skip_hooks: bool,
}

impl Default for SamoyedSettings {
    fn default() -> Self {
        Self {
            hook_directory: default_hook_directory(),
            debug: false,
            fail_fast: default_fail_fast(),
            skip_hooks: false,
        }
    }
}

impl SamoyedSettings {
    /// Check if settings are all default values (for skip_serializing_if)
    fn is_default(&self) -> bool {
        *self == Self::default()
    }
}

impl SamoyedConfig {
    /// Create a default configuration for a specific project type
    pub fn default_for_project_type(project_type: &ProjectType) -> Self {
        let mut hooks = HashMap::new();

        // Add pre-commit hook using the project type's default command
        hooks.insert(
            "pre-commit".to_string(),
            project_type.default_pre_commit_command().to_string(),
        );

        // Add pre-push hook if the project type has a default command
        if let Some(pre_push_cmd) = project_type.default_pre_push_command() {
            hooks.insert("pre-push".to_string(), pre_push_cmd.to_string());
        }

        Self {
            hooks,
            settings: SamoyedSettings::default(),
        }
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), String> {
        // Validate that at least one hook is defined
        if self.hooks.is_empty() {
            return Err("At least one hook must be defined in [hooks] section".to_string());
        }

        // Validate hook names
        for hook_name in self.hooks.keys() {
            if !is_valid_git_hook(hook_name) {
                return Err(format!("Invalid Git hook name: '{hook_name}'"));
            }
        }

        // Validate hook commands are not empty
        for (hook_name, command) in &self.hooks {
            if command.trim().is_empty() {
                return Err(format!("Hook '{hook_name}' cannot have empty command"));
            }
        }

        // Validate hook_directory
        if self.settings.hook_directory.contains("..") {
            return Err("hook_directory cannot contain '..' for security reasons".to_string());
        }

        Ok(())
    }
}

/// Default hook directory
fn default_hook_directory() -> String {
    ".samoyed".to_string()
}

/// Default fail_fast setting
fn default_fail_fast() -> bool {
    true
}

/// Check if a string is a valid Git hook name
fn is_valid_git_hook(name: &str) -> bool {
    matches!(
        name,
        "pre-commit"
            | "pre-merge-commit"
            | "prepare-commit-msg"
            | "commit-msg"
            | "post-commit"
            | "applypatch-msg"
            | "pre-applypatch"
            | "post-applypatch"
            | "pre-rebase"
            | "post-rewrite"
            | "post-checkout"
            | "post-merge"
            | "pre-push"
            | "pre-auto-gc"
    )
}

#[cfg(test)]
#[path = "unit_tests/config_tests.rs"]
mod tests;
