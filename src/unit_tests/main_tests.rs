use super::*;
use crate::environment::mocks::{MockCommandRunner, MockEnvironment, MockFileSystem};
use clap::Parser;

#[test]
fn test_cli_struct_parsing() {
    // Test CLI struct can be created and parsed correctly
    // Test valid arguments
    let args = vec!["samoyed", "init"];
    let cli = Cli::try_parse_from(args);
    assert!(cli.is_ok());

    let parsed = cli.unwrap();
    match parsed.command {
        Some(Commands::Init {
            project_type,
            force: _,
        }) => {
            assert!(project_type.is_none());
        }
        _ => panic!("Expected Init command"),
    }

    // Test with project type argument
    let args = vec!["samoyed", "init", "--project-type", "rust"];
    let cli = Cli::try_parse_from(args);
    assert!(cli.is_ok());

    let parsed = cli.unwrap();
    match parsed.command {
        Some(Commands::Init {
            project_type,
            force: _,
        }) => {
            assert_eq!(project_type, Some("rust".to_string()));
        }
        _ => panic!("Expected Init command"),
    }

    // Test with short form
    let args = vec!["samoyed", "init", "-p", "go"];
    let cli = Cli::try_parse_from(args);
    assert!(cli.is_ok());

    let parsed = cli.unwrap();
    match parsed.command {
        Some(Commands::Init {
            project_type,
            force: _,
        }) => {
            assert_eq!(project_type, Some("go".to_string()));
        }
        _ => panic!("Expected Init command"),
    }
}

#[test]
fn test_cli_no_command() {
    // Test CLI with no command (None case)
    let args = vec!["samoyed"];
    let cli = Cli::try_parse_from(args);
    assert!(cli.is_ok());

    let parsed = cli.unwrap();
    assert!(parsed.command.is_none());
}

#[test]
fn test_cli_invalid_arguments() {
    // Test CLI with invalid arguments
    let args = vec!["samoyed", "invalid-command"];
    let cli = Cli::try_parse_from(args);
    assert!(cli.is_err());
}

#[test]
fn test_cli_hook_command_parsing() {
    // Test Hook command parsing
    let args = vec!["samoyed", "hook", "pre-commit"];
    let cli = Cli::try_parse_from(args);
    assert!(cli.is_ok());

    let parsed = cli.unwrap();
    match parsed.command {
        Some(Commands::Hook { hook_name, args }) => {
            assert_eq!(hook_name, "pre-commit");
            assert!(args.is_empty());
        }
        _ => panic!("Expected Hook command"),
    }

    // Test Hook command with arguments
    let args = vec!["samoyed", "hook", "pre-commit", "arg1", "arg2"];
    let cli = Cli::try_parse_from(args);
    assert!(cli.is_ok());

    let parsed = cli.unwrap();
    match parsed.command {
        Some(Commands::Hook { hook_name, args }) => {
            assert_eq!(hook_name, "pre-commit");
            assert_eq!(args, vec!["arg1", "arg2"]);
        }
        _ => panic!("Expected Hook command"),
    }
}

#[test]
fn test_hook_command_basic_functionality() {
    let env = MockEnvironment::new().with_var("SAMOYED", "1");
    let runner = MockCommandRunner::new();
    let fs = MockFileSystem::new(); // Default has no files/directories

    // Test basic hook execution with no config file and no script
    let args = vec!["samoyed".to_string(), "pre-commit".to_string()];

    // Should exit silently when no script exists (tested via process::exit in the actual code)
    let result = std::panic::catch_unwind(|| hook_command(&env, &runner, &fs, &args));

    // The function calls process::exit(0) when no script exists, which panics in tests
    assert!(result.is_err());
}

#[test]
fn test_hook_command_with_samoyed_zero() {
    let env = MockEnvironment::new().with_var("SAMOYED", "0");
    let runner = MockCommandRunner::new();
    let fs = MockFileSystem::new();

    let args = vec!["samoyed".to_string(), "pre-commit".to_string()];

    // Should exit immediately when SAMOYED=0 (tested via process::exit in the actual code)
    let result = std::panic::catch_unwind(|| hook_command(&env, &runner, &fs, &args));

    // The function calls process::exit(0) when SAMOYED=0, which panics in tests
    assert!(result.is_err());
}

#[test]
fn test_load_hook_command_from_config_success() {
    // Create a mock samoyed.toml file
    let config_content = r#"
[hooks]
pre-commit = "cargo fmt --check"
"#;

    let fs = MockFileSystem::new().with_file("samoyed.toml", config_content);

    let result = load_hook_command_from_config(&fs, "pre-commit", false);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "cargo fmt --check");
}

#[test]
fn test_load_hook_command_from_config_missing_file() {
    let fs = MockFileSystem::new(); // No files by default

    let result = load_hook_command_from_config(&fs, "pre-commit", false);
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("No samoyed.toml configuration file found")
    );
}

#[test]
fn test_load_hook_command_from_config_missing_hook() {
    // Create a mock samoyed.toml file without the requested hook
    let config_content = r#"
[hooks]
pre-push = "cargo test"
"#;

    let fs = MockFileSystem::new().with_file("samoyed.toml", config_content);

    let result = load_hook_command_from_config(&fs, "pre-commit", false);
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("No command configured for hook 'pre-commit'")
    );
}

#[test]
fn test_is_windows_unix_environment() {
    // Test MSYSTEM detection (Git Bash / MSYS2)
    let env = MockEnvironment::new().with_var("MSYSTEM", "MINGW64");
    assert!(is_windows_unix_environment(&env, false));

    let env = MockEnvironment::new().with_var("MSYSTEM", "MSYS");
    assert!(is_windows_unix_environment(&env, false));

    // Test Cygwin detection
    let env = MockEnvironment::new().with_var("CYGWIN", "nodosfilewarning");
    assert!(is_windows_unix_environment(&env, false));

    // Test WSL detection
    let env = MockEnvironment::new().with_var("WSL_DISTRO_NAME", "Ubuntu");
    assert!(is_windows_unix_environment(&env, false));

    let env = MockEnvironment::new().with_var("WSL_INTEROP", "/run/WSL/interop");
    assert!(is_windows_unix_environment(&env, false));

    // Test native Windows (no Unix environment vars)
    let env = MockEnvironment::new();
    assert!(!is_windows_unix_environment(&env, false));
}
