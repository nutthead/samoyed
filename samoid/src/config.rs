//! Configuration structures for samoid.toml
//!
//! Defines the TOML schema and default configurations for different project types.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::project::ProjectType;

/// Main configuration structure for samoid.toml
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SamoidConfig {
    /// Hook definitions (required)
    pub hooks: HashMap<String, String>,
    
    /// Optional settings
    #[serde(default, skip_serializing_if = "SamoidSettings::is_default")]
    pub settings: SamoidSettings,
}

/// Optional settings section for samoid.toml
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct SamoidSettings {
    /// Directory for hook files (default: ".samoid")
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

impl Default for SamoidSettings {
    fn default() -> Self {
        Self {
            hook_directory: default_hook_directory(),
            debug: false,
            fail_fast: default_fail_fast(),
            skip_hooks: false,
        }
    }
}

impl SamoidSettings {
    /// Check if settings are all default values (for skip_serializing_if)
    fn is_default(&self) -> bool {
        *self == Self::default()
    }
}

impl SamoidConfig {
    /// Create a default configuration for a specific project type
    pub fn default_for_project_type(project_type: &ProjectType) -> Self {
        let mut hooks = HashMap::new();
        
        match project_type {
            ProjectType::Rust => {
                hooks.insert("pre-commit".to_string(), "cargo fmt --check && cargo clippy -- -D warnings".to_string());
            }
            ProjectType::Go => {
                hooks.insert("pre-commit".to_string(), "go fmt ./... && go vet ./...".to_string());
            }
            ProjectType::Node => {
                hooks.insert("pre-commit".to_string(), "npm run lint && npm test".to_string());
            }
            ProjectType::Python => {
                hooks.insert("pre-commit".to_string(), "black --check . && flake8".to_string());
            }
            ProjectType::Unknown => {
                hooks.insert("pre-commit".to_string(), "echo 'Please configure your pre-commit hook in samoid.toml'".to_string());
            }
        }
        
        Self {
            hooks,
            settings: SamoidSettings::default(),
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
                return Err(format!("Invalid Git hook name: '{}'", hook_name));
            }
        }
        
        // Validate hook commands are not empty
        for (hook_name, command) in &self.hooks {
            if command.trim().is_empty() {
                return Err(format!("Hook '{}' cannot have empty command", hook_name));
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
    ".samoid".to_string()
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
mod tests {
    use super::*;

    #[test]
    fn test_default_config_rust() {
        let config = SamoidConfig::default_for_project_type(&ProjectType::Rust);
        assert!(config.hooks.contains_key("pre-commit"));
        assert!(config.hooks["pre-commit"].contains("cargo"));
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_default_config_go() {
        let config = SamoidConfig::default_for_project_type(&ProjectType::Go);
        assert!(config.hooks.contains_key("pre-commit"));
        assert!(config.hooks["pre-commit"].contains("go fmt"));
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_default_config_node() {
        let config = SamoidConfig::default_for_project_type(&ProjectType::Node);
        assert!(config.hooks.contains_key("pre-commit"));
        assert!(config.hooks["pre-commit"].contains("npm"));
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_default_config_python() {
        let config = SamoidConfig::default_for_project_type(&ProjectType::Python);
        assert!(config.hooks.contains_key("pre-commit"));
        assert!(config.hooks["pre-commit"].contains("black"));
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validation_empty_hooks() {
        let config = SamoidConfig {
            hooks: HashMap::new(),
            settings: SamoidSettings::default(),
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validation_invalid_hook_name() {
        let mut hooks = HashMap::new();
        hooks.insert("invalid-hook".to_string(), "echo test".to_string());
        
        let config = SamoidConfig {
            hooks,
            settings: SamoidSettings::default(),
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validation_empty_command() {
        let mut hooks = HashMap::new();
        hooks.insert("pre-commit".to_string(), "".to_string());
        
        let config = SamoidConfig {
            hooks,
            settings: SamoidSettings::default(),
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validation_invalid_hook_directory() {
        let mut hooks = HashMap::new();
        hooks.insert("pre-commit".to_string(), "echo test".to_string());
        
        let config = SamoidConfig {
            hooks,
            settings: SamoidSettings {
                hook_directory: "../dangerous".to_string(),
                ..SamoidSettings::default()
            },
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_valid_git_hook_names() {
        assert!(is_valid_git_hook("pre-commit"));
        assert!(is_valid_git_hook("commit-msg"));
        assert!(is_valid_git_hook("pre-push"));
        assert!(!is_valid_git_hook("invalid-hook"));
    }

    #[test]
    fn test_settings_default() {
        let settings = SamoidSettings::default();
        assert_eq!(settings.hook_directory, ".samoid");
        assert!(!settings.debug);
        assert!(settings.fail_fast);
        assert!(!settings.skip_hooks);
    }

    #[test]
    fn test_settings_is_default() {
        let default_settings = SamoidSettings::default();
        assert!(default_settings.is_default());
        
        let custom_settings = SamoidSettings {
            debug: true,
            ..SamoidSettings::default()
        };
        assert!(!custom_settings.is_default());
    }

    #[test]
    fn test_toml_serialization() {
        let config = SamoidConfig::default_for_project_type(&ProjectType::Rust);
        let toml_str = toml::to_string_pretty(&config).unwrap();
        
        // Should contain hooks section
        assert!(toml_str.contains("[hooks]"));
        assert!(toml_str.contains("pre-commit"));
        
        // Should not contain settings section if default
        if config.settings.is_default() {
            assert!(!toml_str.contains("[settings]"));
        }
    }

    #[test]
    fn test_toml_deserialization() {
        let toml_content = r#"
[hooks]
pre-commit = "cargo fmt --check"
pre-push = "cargo test"

[settings]
debug = true
fail_fast = false
"#;
        
        let config: SamoidConfig = toml::from_str(toml_content).unwrap();
        assert_eq!(config.hooks.len(), 2);
        assert!(config.settings.debug);
        assert!(!config.settings.fail_fast);
        assert!(config.validate().is_ok());
    }
}