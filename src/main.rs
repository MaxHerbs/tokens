mod commands;
mod config;
mod oauth;
mod types;

use clap::{Parser, Subcommand};
use commands::{
    CommandContext, CommandHandler, Format, add::AddCommand, delete::DeleteCommand,
    get::GetCommand, list::ListCommand, logout::LogoutCommand,
};
use config::ConfigManager;
use oauth::TokenManager;
use types::ConfigFile;

#[derive(Debug, Parser)]
struct Args {
    #[command(subcommand)]
    cmd: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Retrieve and print an access token
    Get {
        /// Client nickname. Defualts to client ID.
        nickname: String,
        /// Fetch refresh token rather than JWT.
        #[arg(short, long)]
        refresh_token: bool,
        /// Special output formats.
        #[arg(short, long)]
        format: Option<Format>,
        /// Additional scopes. Expects a space-delimitered list.
        #[arg(short, long, num_args = 1.., value_delimiter = ' ')]
        scopes: Vec<String>,
    },
    /// List stored clients.
    List,
    /// Add a new client configuration.
    Add {
        #[arg(short, long)]
        nickname: Option<String>,
        #[arg(short, long)]
        auth_url: String,
        #[arg(short, long)]
        client_id: String,
        #[arg(short, long)]
        secret: Option<String>,
    },
    /// Remove a saved client.
    Delete { nickname: String },
    /// Logout of client.
    Logout { nickname: String },
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let config_manager = ConfigManager::new();
    let token_manager = TokenManager::new();
    let config_path = config_manager.get_config_path();
    let mut config = config_manager.read_config(&config_path).unwrap_or_default();

    if let Err(e) = run_command(args, &mut config, &config_manager, &token_manager).await {
        eprintln!("{e}");
    }
}

async fn run_command(
    args: Args,
    config: &mut ConfigFile,
    config_manager: &ConfigManager,
    token_manager: &TokenManager,
) -> Result<(), Box<dyn std::error::Error>> {
    let context = CommandContext {
        config,
        config_manager,
        token_manager,
    };

    match args.cmd {
        Command::Add {
            nickname,
            auth_url,
            client_id,
            secret,
        } => {
            let command = AddCommand {
                nickname,
                auth_url,
                client_id,
                secret,
            };
            command.execute(context).await
        }
        Command::List => {
            let command = ListCommand;
            command.execute(context).await
        }
        Command::Get {
            nickname,
            refresh_token,
            format,
            scopes,
        } => {
            let command = GetCommand {
                nickname,
                refresh_token,
                format,
                scopes,
            };
            command.execute(context).await
        }
        Command::Delete { nickname } => {
            let command = DeleteCommand { nickname };
            command.execute(context).await
        }
        Command::Logout { nickname } => {
            let command = LogoutCommand { nickname };
            command.execute(context).await
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::config::ConfigManager;
    use crate::oauth::TokenManager;
    use crate::types::{AuthConfig, ConfigFile};
    use crate::{Args, Command, run_command};
    use std::collections::HashMap;
    use tempfile::TempDir;

    #[tokio::test]
    async fn add_client() {
        let command = Command::Add {
            nickname: Some("test_name".to_string()),
            auth_url: "domain".to_string(),
            client_id: "clientId".to_string(),
            secret: None,
        };

        let _tmp_dir = TempDir::new().unwrap();
        let mut config = HashMap::new();
        config.insert(
            "prod".to_string(),
            AuthConfig {
                auth_url: "existing".to_string(),
                client_id: "existing".to_string(),
                refresh_token: None,
                secret: None,
            },
        );
        let mut config_file = ConfigFile { clients: config };

        let config_manager = ConfigManager::new();
        let token_manager = TokenManager::new();
        let args = crate::Args { cmd: command };

        let result = run_command(args, &mut config_file, &config_manager, &token_manager).await;
        assert!(result.is_ok());

        // Verify the client was added to the config
        assert!(config_file.clients.contains_key("test_name"));
        assert!(config_file.clients.contains_key("prod"));

        let existing = config_file.clients.get("prod").unwrap();
        let new = config_file.clients.get("test_name").unwrap();
        assert!(existing.auth_url == "existing");
        assert!(new.auth_url == "domain");
    }

    #[tokio::test]
    async fn test_run_command_list() {
        let mut config = ConfigFile {
            clients: {
                let mut clients = HashMap::new();
                clients.insert(
                    "test_client".to_string(),
                    AuthConfig {
                        auth_url: "https://example.com".to_string(),
                        client_id: "client123".to_string(),
                        refresh_token: Some("refresh123".to_string()),
                        secret: None,
                    },
                );
                clients
            },
        };

        let config_manager = ConfigManager::new();
        let token_manager = TokenManager::new();
        let args = Args { cmd: Command::List };

        let result = run_command(args, &mut config, &config_manager, &token_manager).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_run_command_delete_existing() {
        let mut config = ConfigFile {
            clients: {
                let mut clients = HashMap::new();
                clients.insert(
                    "test_client".to_string(),
                    AuthConfig {
                        auth_url: "https://example.com".to_string(),
                        client_id: "client123".to_string(),
                        refresh_token: Some("refresh123".to_string()),
                        secret: None,
                    },
                );
                clients
            },
        };

        let config_manager = ConfigManager::new();
        let token_manager = TokenManager::new();
        let args = Args {
            cmd: Command::Delete {
                nickname: "test_client".to_string(),
            },
        };

        let result = run_command(args, &mut config, &config_manager, &token_manager).await;
        assert!(result.is_ok());
        assert!(!config.clients.contains_key("test_client"));
    }

    #[tokio::test]
    async fn test_run_command_delete_nonexistent() {
        let mut config = ConfigFile::default();

        let config_manager = ConfigManager::new();
        let token_manager = TokenManager::new();
        let args = Args {
            cmd: Command::Delete {
                nickname: "nonexistent".to_string(),
            },
        };

        let result = run_command(args, &mut config, &config_manager, &token_manager).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_run_command_logout() {
        let mut config = ConfigFile {
            clients: {
                let mut clients = HashMap::new();
                clients.insert(
                    "test_client".to_string(),
                    AuthConfig {
                        auth_url: "https://example.com".to_string(),
                        client_id: "client123".to_string(),
                        refresh_token: Some("refresh123".to_string()),
                        secret: None,
                    },
                );
                clients
            },
        };

        let config_manager = ConfigManager::new();
        let token_manager = TokenManager::new();
        let args = Args {
            cmd: Command::Logout {
                nickname: "test_client".to_string(),
            },
        };

        let result = run_command(args, &mut config, &config_manager, &token_manager).await;
        assert!(result.is_ok());

        // Verify refresh token was cleared
        let client = config.clients.get("test_client").unwrap();
        assert_eq!(client.refresh_token, None);
    }

    #[tokio::test]
    async fn test_run_command_get_nonexistent_client() {
        let mut config = ConfigFile::default();

        let config_manager = ConfigManager::new();
        let token_manager = TokenManager::new();
        let args = Args {
            cmd: Command::Get {
                nickname: "nonexistent".to_string(),
                refresh_token: false,
                format: None,
                scopes: vec![],
            },
        };

        let result = run_command(args, &mut config, &config_manager, &token_manager).await;
        assert!(result.is_ok()); // Should succeed but print error message
    }

    #[tokio::test]
    async fn test_run_command_get_with_scopes_and_format() {
        let mut config = ConfigFile::default();

        let config_manager = ConfigManager::new();
        let token_manager = TokenManager::new();
        let args = Args {
            cmd: Command::Get {
                nickname: "test_client".to_string(),
                refresh_token: true,
                format: Some(crate::commands::Format::Header),
                scopes: vec!["openid".to_string(), "profile".to_string()],
            },
        };

        let result = run_command(args, &mut config, &config_manager, &token_manager).await;
        assert!(result.is_ok());
    }
}
