mod tokens;
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
    Get { nickname: String },
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
    let mut config = read_config(&config_path).unwrap_or_default();

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
            if let Err(err) = save_config(&config_path, &config) {
                eprintln!("Failed to save config: {}", err);
            } else {
                println!("Client '{}' added.", nickname);
            }
        }

        Command::List => {
            let table = list_clients(&mut config);
            table.printstd();
        }

        Command::Get { nickname } => {
            if let Some(auth) = config.clients.get_mut(&nickname) {
                match get_or_refresh_token_with_input(auth, prompt_credentials_from_user).await {
                    Ok(token) => {
                        println!("{}", token);
                        if let Err(err) = save_config(&config_path, &config) {
                            eprintln!(
                                "Warning: token retrieved but failed to save config: {}",
                                err
                            );
                        }
                    }
                    Err(err) => {
                        eprintln!("Failed to retrieve token: {}", err);
                    }
                }
            } else {
                eprintln!("Client '{}' not found.", nickname);
            }
        }

        Command::Delete { nickname } => {
            delete_client(&nickname, &mut config);
        }
    }
}
