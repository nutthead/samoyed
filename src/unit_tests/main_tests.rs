use super::*;
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
