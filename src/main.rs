mod tokens;
use std::path::Path;

use clap::{Parser, Subcommand};
use tokens::{
    AuthConfig, get_config_path, get_or_refresh_token_with_input, prompt_credentials_from_user,
    read_config, save_config,
};

use crate::tokens::{delete_client, list_clients};

#[derive(Debug, Parser)]
struct Args {
    #[command(subcommand)]
    cmd: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Retrieve and print an access token
    Get {
        nickname: String,
        #[arg(short, long)]
        refresh_token: bool,
    },
    /// List stored clients
    List,
    /// Add a new client configuration
    Add {
        #[arg(short, long)]
        nickname: Option<String>,
        #[arg(short, long)]
        auth_url: String,
        #[arg(short, long)]
        client_id: String,
    },
    /// Remove a saved client
    Delete { nickname: String },
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let config_path = get_config_path();
    let config = read_config(&config_path).unwrap_or_default();
    run_command(args, config, &config_path, prompt_credentials_from_user).await;
}

async fn run_command<F>(
    args: Args,
    mut config: tokens::ConfigFile,
    config_path: &Path,
    prompt_fn: F,
) where
    F: Fn() -> Result<(String, String), Box<dyn std::error::Error>>,
{
    match args.cmd {
        Command::Add {
            nickname,
            auth_url,
            client_id,
        } => {
            let nickname = match nickname {
                Some(nickname) => nickname,
                None => client_id.clone(),
            };

            config.clients.insert(
                nickname.clone(),
                AuthConfig {
                    auth_url,
                    client_id,
                    refresh_token: None,
                },
            );
            if let Err(err) = save_config(config_path, &config) {
                eprintln!("Failed to save config: {err}");
            } else {
                println!("Client '{nickname}' added.");
            }
        }

        Command::List => {
            let table = list_clients(&mut config);
            table.printstd();
        }

        Command::Get {
            nickname,
            refresh_token,
        } => {
            if let Some(auth) = config.clients.get_mut(&nickname) {
                match get_or_refresh_token_with_input(auth, refresh_token, prompt_fn).await {
                    Ok(token) => {
                        println!("{token}");
                        if let Err(err) = save_config(config_path, &config) {
                            eprintln!(
                                "Warning: token retrieved but failed to save config: {err}"
                            );
                        }
                    }
                    Err(err) => {
                        eprintln!("Failed to retrieve token: {err}");
                    }
                }
            } else {
                eprintln!("Client '{nickname}' not found.");
            }
        }

        Command::Delete { nickname } => {
            delete_client(&nickname, &mut config);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use mockito::Server;
    use tempfile::TempDir;

    use crate::{Command, run_command, tokens::AuthConfig, tokens::ConfigFile};

    #[tokio::test]
    async fn get_new_client() {
        let mock_response = r#"{"access_token": "token", "refresh_token": "refresh"}"#;
        let mut server = Server::new_async().await;
        let mock = server
            .mock("POST", "/realms/master/protocol/openid-connect/token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(mock_response)
            .create_async()
            .await;

        let tmp_dir = TempDir::new().unwrap();
        let conf_dir = tmp_dir.path().join("config.json");
        let mut config = HashMap::new();
        config.insert(
            "prod".to_string(),
            AuthConfig {
                auth_url: format!("{}/realms/master", server.url()),
                client_id: "test".to_string(),
                refresh_token: None,
            },
        );
        let config_file = ConfigFile { clients: config };

        let command = Command::Get {
            nickname: "prod".to_string(),
            refresh_token: false,
        };
        let args = crate::Args { cmd: command };
        run_command(args, config_file, &conf_dir, input).await;
        mock.assert_async().await;

        println!("{:?}", tmp_dir.path().read_dir());
        let response_config: ConfigFile =
            serde_json::from_reader(std::fs::File::open(&conf_dir).unwrap()).unwrap();

        let client = response_config.clients.get("prod").unwrap();
        assert_eq!(client.client_id, "test");
        assert_eq!(client.refresh_token.as_deref(), Some("refresh"));
    }

    #[tokio::test]
    async fn add_client() {
        let command = Command::Add {
            nickname: Some("test_name".to_string()),
            auth_url: "domain".to_string(),
            client_id: "clientId".to_string(),
        };

        let tmp_dir = TempDir::new().unwrap();
        let conf_dir = tmp_dir.path().join("config.json");
        let mut config = HashMap::new();
        config.insert(
            "prod".to_string(),
            AuthConfig {
                auth_url: "existing".to_string(),
                client_id: "existing".to_string(),
                refresh_token: None,
            },
        );
        let config_file = ConfigFile { clients: config };

        let args = crate::Args { cmd: command };
        run_command(args, config_file, &conf_dir, input).await;

        println!("{:?}", tmp_dir.path().read_dir());
        let read_back_config: ConfigFile =
            serde_json::from_reader(std::fs::File::open(&conf_dir).unwrap()).unwrap();

        let existing = read_back_config.clients.get("prod").unwrap();
        let new = read_back_config.clients.get("test_name").unwrap();
        assert!(existing.auth_url == "existing");
        assert!(new.auth_url == "domain");
    }

    fn input() -> Result<(String, String), Box<dyn std::error::Error>> {
        Ok(("user".to_string(), "password".to_string()))
    }
}
