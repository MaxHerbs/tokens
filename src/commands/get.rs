use super::{CommandContext, CommandHandler, Format};
use serde_json::json;
use std::error::Error;

pub struct GetCommand {
    pub nickname: String,
    pub refresh_token: bool,
    pub format: Option<Format>,
    pub scopes: Vec<String>,
}

impl CommandHandler for GetCommand {
    async fn execute(&self, context: CommandContext<'_>) -> Result<(), Box<dyn Error>> {
        if let Some(auth) = context
            .config_manager
            .get_client_mut(context.config, &self.nickname)
        {
            let credentials_provider = context.credentials_provider;

            match context
                .token_manager
                .get_or_refresh_token(auth, self.refresh_token, &self.scopes, credentials_provider)
                .await
            {
                Ok(token) => {
                    let msg = if let Some(ref format) = self.format {
                        match format {
                            Format::Header => json!({
                                "Authorization": format!("Bearer {token}")
                            })
                            .to_string(),
                        }
                    } else {
                        token
                    };
                    println!("{msg}");

                    let config_path = context.config_manager.get_config_path();
                    if let Err(err) = context
                        .config_manager
                        .save_config(&config_path, context.config)
                    {
                        eprintln!("Warning: token retrieved but failed to save config: {err}");
                    }
                }
                Err(err) => {
                    eprintln!("Failed to retrieve token: {err}");
                }
            }
        } else {
            eprintln!("Client '{}' not found.", self.nickname);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        config::ConfigManager,
        oauth::TokenManager,
        types::{AuthConfig, CredentialsProvider},
    };
    use httpmock::{Method::POST, MockServer};

    struct MockCredentialsProvider;

    impl CredentialsProvider for MockCredentialsProvider {
        fn get_credentials(&self) -> Result<(String, String), Box<dyn std::error::Error>> {
            Ok(("user".into(), "pass".into()))
        }
    }

    mod get_command_tests {
        use super::*;
        use crate::commands::get::GetCommand;
        use crate::commands::{CommandContext, CommandHandler, Format};
        use crate::types::ConfigFile;
        use std::collections::HashMap;
        use tokio;

        #[tokio::test]
        async fn test_get_command_client_not_found() {
            let config = ConfigFile {
                clients: HashMap::new(),
            };
            let mut config = config;
            let config_manager = ConfigManager::new();
            let token_manager = TokenManager::new();
            let mock_credentials_provider = MockCredentialsProvider;

            let context = CommandContext {
                config: &mut config,
                config_manager: &config_manager,
                token_manager: &token_manager,
                credentials_provider: &mock_credentials_provider,
            };

            let get_command = GetCommand {
                nickname: "nonexistent".to_string(),
                refresh_token: false,
                format: None,
                scopes: vec![],
            };

            let result = get_command.execute(context).await;
            assert!(result.is_ok());
        }

        #[tokio::test]
        async fn test_get_command_token_success_no_format() {
            let server = MockServer::start();

            let token_response = serde_json::json!({
                "access_token": "test_access_token",
                "refresh_token": "test_refresh_token"
            });

            let _mock = server.mock(|when, then| {
                when.method(POST).path("/protocol/openid-connect/token");
                then.status(200)
                    .header("content-type", "application/json")
                    .json_body(token_response.clone());
            });

            let mut clients = HashMap::new();
            clients.insert(
                "test_client".into(),
                AuthConfig {
                    auth_url: server.url(""),
                    client_id: "test-client".into(),
                    refresh_token: Some("existing_refresh_token".into()),
                    secret: None,
                },
            );

            let mut config = ConfigFile { clients };
            let config_manager = ConfigManager::new();
            let token_manager = TokenManager::new();
            let mock_credentials_provider = MockCredentialsProvider;

            let context = CommandContext {
                config: &mut config,
                config_manager: &config_manager,
                token_manager: &token_manager,
                credentials_provider: &mock_credentials_provider,
            };

            let get_command = GetCommand {
                nickname: "test_client".to_string(),
                refresh_token: false,
                format: None,
                scopes: vec![],
            };

            let result = get_command.execute(context).await;
            assert!(result.is_ok());
        }

        #[tokio::test]
        async fn test_get_command_token_success_with_header_format() {
            let server = MockServer::start();

            let token_response = serde_json::json!({
                "access_token": "test_access_token",
                "refresh_token": "test_refresh_token"
            });

            let _mock = server.mock(|when, then| {
                when.method(POST).path("/protocol/openid-connect/token");
                then.status(200)
                    .header("content-type", "application/json")
                    .json_body(token_response.clone());
            });

            let mut clients = HashMap::new();
            clients.insert(
                "test_client".into(),
                AuthConfig {
                    auth_url: server.url(""),
                    client_id: "test-client".into(),
                    refresh_token: Some("existing_refresh_token".into()),
                    secret: None,
                },
            );

            let mut config = ConfigFile { clients };
            let config_manager = ConfigManager::new();
            let token_manager = TokenManager::new();
            let mock_credentials_provider = MockCredentialsProvider;

            let context = CommandContext {
                config: &mut config,
                config_manager: &config_manager,
                token_manager: &token_manager,
                credentials_provider: &mock_credentials_provider,
            };

            let get_command = GetCommand {
                nickname: "test_client".to_string(),
                refresh_token: false,
                format: Some(Format::Header),
                scopes: vec![],
            };

            let result = get_command.execute(context).await;
            assert!(result.is_ok());
        }

        #[tokio::test]
        async fn test_get_command_token_failure() {
            let server = MockServer::start();

            let _mock = server.mock(|when, then| {
                when.method(POST).path("/protocol/openid-connect/token");
                then.status(400);
            });

            let mut clients = HashMap::new();
            clients.insert(
                "test_client".into(),
                AuthConfig {
                    auth_url: server.url(""),
                    client_id: "test-client".into(),
                    refresh_token: Some("invalid_refresh_token".into()),
                    secret: None,
                },
            );

            let mut config = ConfigFile { clients };
            let config_manager = ConfigManager::new();
            let token_manager = TokenManager::new();
            let mock_credentials_provider = MockCredentialsProvider;

            let context = CommandContext {
                config: &mut config,
                config_manager: &config_manager,
                token_manager: &token_manager,
                credentials_provider: &mock_credentials_provider,
            };

            let get_command = GetCommand {
                nickname: "test_client".to_string(),
                refresh_token: false,
                format: None,
                scopes: vec![],
            };

            let result = get_command.execute(context).await;
            assert!(result.is_ok());
        }

        #[tokio::test]
        async fn test_get_command_with_refresh_token_flag() {
            let server = MockServer::start();

            let token_response = serde_json::json!({
                "access_token": "test_access_token",
                "refresh_token": "test_refresh_token"
            });

            let _mock = server.mock(|when, then| {
                when.method(POST)
                    .path("/protocol/openid-connect/token")
                    .body_contains("grant_type=refresh_token");
                then.status(200)
                    .header("content-type", "application/json")
                    .json_body(token_response.clone());
            });

            let mut clients = HashMap::new();
            clients.insert(
                "test_client".into(),
                AuthConfig {
                    auth_url: server.url(""),
                    client_id: "test-client".into(),
                    refresh_token: Some("existing_refresh_token".into()),
                    secret: None,
                },
            );

            let mut config = ConfigFile { clients };
            let config_manager = ConfigManager::new();
            let token_manager = TokenManager::new();
            let mock_credentials_provider = MockCredentialsProvider;

            let context = CommandContext {
                config: &mut config,
                config_manager: &config_manager,
                token_manager: &token_manager,
                credentials_provider: &mock_credentials_provider,
            };

            let get_command = GetCommand {
                nickname: "test_client".to_string(),
                refresh_token: true,
                format: None,
                scopes: vec![],
            };

            let result = get_command.execute(context).await;
            assert!(result.is_ok());
        }

        #[tokio::test]
        async fn test_get_command_with_scopes() {
            let server = MockServer::start();

            let token_response = serde_json::json!({
                "access_token": "test_access_token",
                "refresh_token": "test_refresh_token"
            });

            let _mock = server.mock(|when, then| {
                when.method(POST).path("/protocol/openid-connect/token");
                then.status(200)
                    .header("content-type", "application/json")
                    .json_body(token_response.clone());
            });

            let mut clients = HashMap::new();
            clients.insert(
                "test_client".into(),
                AuthConfig {
                    auth_url: server.url(""),
                    client_id: "test-client".into(),
                    refresh_token: Some("existing_refresh_token".into()),
                    secret: None,
                },
            );

            let mut config = ConfigFile { clients };
            let config_manager = ConfigManager::new();
            let token_manager = TokenManager::new();
            let mock_credentials_provider = MockCredentialsProvider;

            let context = CommandContext {
                config: &mut config,
                config_manager: &config_manager,
                token_manager: &token_manager,
                credentials_provider: &mock_credentials_provider,
            };

            let get_command = GetCommand {
                nickname: "test_client".to_string(),
                refresh_token: false,
                format: None,
                scopes: vec!["read".to_string(), "write".to_string()],
            };

            let result = get_command.execute(context).await;
            assert!(result.is_ok());
        }
    }
}
