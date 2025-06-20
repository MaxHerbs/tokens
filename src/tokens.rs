use prettytable::{Table, row};
use reqwest::Client;
use rpassword::read_password;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
};

const CONFIG_DIR: &str = ".config/tokens";
const CONFIG_FILE: &str = "config.json";

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct AuthConfig {
    pub auth_url: String,
    pub client_id: String,
    pub refresh_token: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default, PartialEq)]
pub struct ConfigFile {
    pub clients: HashMap<String, AuthConfig>,
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
    refresh_token: Option<String>,
}

pub async fn get_or_refresh_token_with_input<F>(
    auth: &mut AuthConfig,
    fetch_refresh_token: bool,
    prompt_credentials: F,
) -> Result<String, Box<dyn std::error::Error>>
where
    F: Fn() -> Result<(String, String), Box<dyn std::error::Error>>,
{
    let client = Client::new();

    if let Some(refresh_token) = auth.refresh_token.clone() {
        if let Ok(token) = use_refresh_token(&client, auth, &refresh_token).await {
            if fetch_refresh_token {
                return Ok(refresh_token);
            } else {
                return Ok(token);
            }
        }
    }

    let (username, password) = prompt_credentials()?;
    request_new_token(&client, auth, &username, &password).await
}

async fn request_new_token(
    client: &Client,
    auth: &mut AuthConfig,
    username: &str,
    password: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let form = [
        ("grant_type", "password"),
        ("client_id", &auth.client_id),
        ("username", username),
        ("password", password),
    ];

    let url = format!("{}/protocol/openid-connect/token", auth.auth_url);
    let res = client
        .post(&url)
        .form(&form)
        .send()
        .await?
        .error_for_status()?;

    let data: TokenResponse = res.json().await?;
    if let Some(refresh) = &data.refresh_token {
        auth.refresh_token = Some(refresh.clone());
    }

    Ok(data.access_token)
}

pub fn prompt_credentials_from_user() -> Result<(String, String), Box<dyn std::error::Error>> {
    print!("Username: ");
    io::stdout().flush()?;
    let mut username = String::new();
    io::stdin().read_line(&mut username)?;
    let username = username.trim().to_string();

    print!("Password: ");
    io::stdout().flush()?;
    let password = read_password()?;

    Ok((username, password))
}

async fn use_refresh_token(
    client: &Client,
    auth: &mut AuthConfig,
    refresh_token: &str,
) -> Result<String, reqwest::Error> {
    let form = [
        ("grant_type", "refresh_token"),
        ("client_id", &auth.client_id),
        ("refresh_token", refresh_token),
    ];
    let url = format!("{}/protocol/openid-connect/token", auth.auth_url);

    let res = client
        .post(&url)
        .form(&form)
        .send()
        .await?
        .error_for_status()?;
    let data: TokenResponse = res.json().await?;

    if let Some(refresh) = &data.refresh_token {
        auth.refresh_token = Some(refresh.clone());
    }

    Ok(data.access_token)
}

pub fn get_config_path() -> PathBuf {
    let home = dirs::home_dir().expect("Could not determine home directory.");
    home.join(CONFIG_DIR).join(CONFIG_FILE)
}

pub fn read_config(path: &Path) -> Option<ConfigFile> {
    if path.exists() {
        fs::read_to_string(path)
            .ok()
            .and_then(|data| serde_json::from_str(&data).ok())
    } else {
        None
    }
}

pub fn save_config(path: &Path, config: &ConfigFile) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let data = serde_json::to_string_pretty(config)?;
    fs::write(path, data)
}

pub fn delete_client(nickname: &str, config: &mut ConfigFile) {
    if config.clients.remove(nickname).is_some() {
        let config_file = get_config_path();
        match save_config(&config_file, config) {
            Ok(()) => println!("Client '{}' removed", nickname),
            Err(e) => println!("Failed to update config file.\n{}", e),
        }
    } else {
        println!("Client '{}' doesn't exist", nickname);
    }
}

pub fn list_clients(config: &mut ConfigFile) -> Table {
    let mut table = Table::new();
    table.add_row(row!["Nickname", "ClientId"]);
    config.clients.iter().for_each(|(nickname, config)| {
        table.add_row(row![nickname, &config.client_id]);
    });
    table
}
