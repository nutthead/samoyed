use super::*;

#[test]
fn default_config_rust() {
    let config = SamoyedConfig::default_for_project_type(&ProjectType::Rust);
    assert!(config.hooks.contains_key("pre-commit"));
    assert!(config.hooks["pre-commit"].contains("cargo"));
    assert!(config.validate().is_ok());
}

#[test]
fn default_config_go() {
    let config = SamoyedConfig::default_for_project_type(&ProjectType::Go);
    assert!(config.hooks.contains_key("pre-commit"));
    assert!(config.hooks["pre-commit"].contains("go fmt"));
    assert!(config.validate().is_ok());
}

#[test]
fn default_config_node() {
    let config = SamoyedConfig::default_for_project_type(&ProjectType::Node);
    assert!(config.hooks.contains_key("pre-commit"));
    assert!(config.hooks["pre-commit"].contains("npm"));
    assert!(config.validate().is_ok());
}

#[test]
fn default_config_python() {
    let config = SamoyedConfig::default_for_project_type(&ProjectType::Python);
    assert!(config.hooks.contains_key("pre-commit"));
    assert!(config.hooks["pre-commit"].contains("black"));
    assert!(config.validate().is_ok());
}

#[test]
fn validation_rejects_empty_hooks() {
    let config = SamoyedConfig {
        hooks: HashMap::new(),
        settings: SamoyedSettings::default(),
    };
    assert!(config.validate().is_err());
}

#[test]
fn validation_rejects_invalid_hook_names() {
    let mut hooks = HashMap::new();
    hooks.insert("invalid-hook".to_string(), "echo test".to_string());

    let config = SamoyedConfig {
        hooks,
        settings: SamoyedSettings::default(),
    };
    assert!(config.validate().is_err());
}

#[test]
fn validation_rejects_empty_commands() {
    let mut hooks = HashMap::new();
    hooks.insert("pre-commit".to_string(), "".to_string());

    let config = SamoyedConfig {
        hooks,
        settings: SamoyedSettings::default(),
    };
    assert!(config.validate().is_err());
}

#[test]
fn validation_rejects_invalid_hook_directory() {
    let mut hooks = HashMap::new();
    hooks.insert("pre-commit".to_string(), "echo test".to_string());

    let config = SamoyedConfig {
        hooks,
        settings: SamoyedSettings {
            hook_directory: "../dangerous".to_string(),
            ..SamoyedSettings::default()
        },
    };
    assert!(config.validate().is_err());
}

#[test]
fn valid_git_hook_names() {
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
fn settings_default() {
    let settings = SamoyedSettings::default();
    assert_eq!(settings.hook_directory, ".samoyed");
    assert!(!settings.debug);
    assert!(settings.fail_fast);
    assert!(!settings.skip_hooks);
}

#[test]
fn settings_is_default() {
    let default_settings = SamoyedSettings::default();
    assert!(default_settings.is_default());

    let custom_settings = SamoyedSettings {
        debug: true,
        ..SamoyedSettings::default()
    };
    assert!(!custom_settings.is_default());
}

#[test]
fn toml_serialization() {
    let config = SamoyedConfig::default_for_project_type(&ProjectType::Rust);
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
fn toml_deserialization() {
    let toml_content = r#"
[hooks]
pre-commit = "cargo fmt --check"
pre-push = "cargo test"

[settings]
debug = true
fail_fast = false
"#;

    let config: SamoyedConfig = toml::from_str(toml_content).unwrap();
    assert_eq!(config.hooks.len(), 2);
    assert!(config.settings.debug);
    assert!(!config.settings.fail_fast);
    assert!(config.validate().is_ok());
}

#[test]
fn validate_method_coverage() {
    // Test various validation scenarios to ensure complete coverage
    let mut config = SamoyedConfig::default_for_project_type(&ProjectType::Rust);

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
fn comprehensive_validation_coverage() {
    // Test all validation code paths to ensure 100% coverage
    let mut config = SamoyedConfig {
        hooks: std::collections::HashMap::new(),
        settings: SamoyedSettings::default(),
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
        let mut test_config = SamoyedConfig {
            hooks: std::collections::HashMap::new(),
            settings: SamoyedSettings::default(),
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
fn settings_validation_edge_cases() {
    // Test edge cases in settings validation
    let mut config = SamoyedConfig::default_for_project_type(&ProjectType::Rust);

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
fn default_functions_coverage() {
    // Test the default functions to ensure they're covered
    let default_dir = super::default_hook_directory();
    assert_eq!(default_dir, ".samoyed");

    let default_fail_fast = super::default_fail_fast();
    assert!(default_fail_fast);

    // Test Settings construction with defaults
    let settings = SamoyedSettings {
        hook_directory: default_hook_directory(),
        debug: false,
        fail_fast: default_fail_fast,
        skip_hooks: false,
    };
    assert_eq!(settings.hook_directory, ".samoyed");
    assert!(settings.fail_fast);
}

#[test]
fn all_project_type_configs() {
    // Test that all project types can generate configs and validate successfully
    for project_type in [
        ProjectType::Rust,
        ProjectType::Go,
        ProjectType::Node,
        ProjectType::Python,
        ProjectType::Unknown,
    ] {
        let config = SamoyedConfig::default_for_project_type(&project_type);
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
