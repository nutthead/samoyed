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
        // Test all valid Git hook names
        assert!(is_valid_git_hook("pre-commit"));
        assert!(is_valid_git_hook("pre-merge-commit"));
        assert!(is_valid_git_hook("prepare-commit-msg"));
        assert!(is_valid_git_hook("commit-msg"));
        assert!(is_valid_git_hook("post-commit"));
        assert!(is_valid_git_hook("applypatch-msg"));
        assert!(is_valid_git_hook("pre-applypatch"));
        assert!(is_valid_git_hook("post-applypatch"));
        assert!(is_valid_git_hook("pre-rebase"));
        assert!(is_valid_git_hook("post-rewrite"));
        assert!(is_valid_git_hook("post-checkout"));
        assert!(is_valid_git_hook("post-merge"));
        assert!(is_valid_git_hook("pre-push"));
        assert!(is_valid_git_hook("pre-auto-gc"));

        // Test invalid hook names
        assert!(!is_valid_git_hook("invalid-hook"));
        assert!(!is_valid_git_hook(""));
        assert!(!is_valid_git_hook("custom-hook"));
        assert!(!is_valid_git_hook("pre-unknown"));
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

    #[test]
    fn test_validate_method_coverage() {
        // Test various validation scenarios to ensure complete coverage
        let mut config = SamoidConfig::default_for_project_type(&ProjectType::Rust);

        // Test valid configuration
        assert!(config.validate().is_ok());

        // Test validation with custom settings that should pass
        config.settings.debug = true;
        config.settings.skip_hooks = true;
        config.settings.fail_fast = false;
        assert!(config.validate().is_ok());

        // Test validation ensures proper error handling paths are covered
        config.settings.hook_directory = "valid-dir".to_string();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_comprehensive_validation_coverage() {
        // Test all validation code paths to ensure 100% coverage
        let mut config = SamoidConfig {
            hooks: std::collections::HashMap::new(),
            settings: SamoidSettings::default(),
        };

        // Add a valid hook to test hook validation logic
        config
            .hooks
            .insert("pre-commit".to_string(), "echo test".to_string());
        assert!(config.validate().is_ok());

        // Test with multiple hooks to cover iteration logic
        config
            .hooks
            .insert("post-commit".to_string(), "echo post".to_string());
        config
            .hooks
            .insert("prepare-commit-msg".to_string(), "echo prepare".to_string());
        assert!(config.validate().is_ok());

        // Test hook command trimming logic
        config
            .hooks
            .insert("pre-push".to_string(), "  echo with spaces  ".to_string());
        assert!(config.validate().is_ok());

        // Test validation with all valid hook types to ensure complete coverage
        for hook_name in [
            "pre-commit",
            "pre-merge-commit",
            "prepare-commit-msg",
            "commit-msg",
            "post-commit",
            "applypatch-msg",
            "pre-applypatch",
            "post-applypatch",
            "pre-rebase",
            "post-rewrite",
            "post-checkout",
            "post-merge",
            "pre-push",
            "pre-auto-gc",
        ] {
            let mut test_config = SamoidConfig {
                hooks: std::collections::HashMap::new(),
                settings: SamoidSettings::default(),
            };
            test_config
                .hooks
                .insert(hook_name.to_string(), "test command".to_string());
            assert!(
                test_config.validate().is_ok(),
                "Failed validation for hook: {hook_name}"
            );
        }
    }

    #[test]
    fn test_settings_validation_edge_cases() {
        // Test edge cases in settings validation
        let mut config = SamoidConfig::default_for_project_type(&ProjectType::Rust);

        // Test hook_directory validation with valid paths
        config.settings.hook_directory = ".custom".to_string();
        assert!(config.validate().is_ok());

        config.settings.hook_directory = "hooks-dir".to_string();
        assert!(config.validate().is_ok());

        config.settings.hook_directory = "very/deep/nested/path".to_string();
        assert!(config.validate().is_ok());

        // Test with all boolean combinations
        config.settings.debug = true;
        config.settings.fail_fast = false;
        config.settings.skip_hooks = true;
        assert!(config.validate().is_ok());

        config.settings.debug = false;
        config.settings.fail_fast = true;
        config.settings.skip_hooks = false;
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_default_functions_coverage() {
        // Test the default functions to ensure they're covered
        let default_dir = super::default_hook_directory();
        assert_eq!(default_dir, ".samoid");

        let default_fail_fast = super::default_fail_fast();
        assert!(default_fail_fast);

        // Test Settings construction with defaults
        let settings = SamoidSettings {
            hook_directory: default_hook_directory(),
            debug: false,
            fail_fast: default_fail_fast,
            skip_hooks: false,
        };
        assert_eq!(settings.hook_directory, ".samoid");
        assert!(settings.fail_fast);
    }

    #[test]
    fn test_all_project_type_configs() {
        // Test that all project types can generate configs and validate successfully
        for project_type in [
            ProjectType::Rust,
            ProjectType::Go,
            ProjectType::Node,
            ProjectType::Python,
            ProjectType::Unknown,
        ] {
            let config = SamoidConfig::default_for_project_type(&project_type);
            assert!(
                config.validate().is_ok(),
                "Failed validation for project type: {project_type:?}"
            );
            assert!(
                !config.hooks.is_empty(),
                "No hooks generated for project type: {project_type:?}"
            );
        }
    }
}
