use super::{CommandContext, CommandHandler, Format};
use crate::types::ConsoleCredentialsProvider;
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
            let credentials_provider = ConsoleCredentialsProvider;

            match context
                .token_manager
                .get_or_refresh_token(
                    auth,
                    self.refresh_token,
                    &self.scopes,
                    &credentials_provider,
                )
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
        types::{AuthConfig, ConfigFile, CredentialsProvider},
    };
    use httpmock::{Method::POST, MockServer};
    use std::{collections::HashMap, path::Path};
    use tempfile::tempdir;

    struct MockCredentialsProvider;

    impl CredentialsProvider for MockCredentialsProvider {
        fn get_credentials(&self) -> Result<(String, String), Box<dyn std::error::Error>> {
            Ok(("user".into(), "pass".into()))
        }
    }

    #[tokio::test]
    async fn test_get_or_refresh_token_with_valid_refresh_token() {
        let server = MockServer::start();

        let token_response = serde_json::json!({
            "access_token": "new_access_token",
            "refresh_token": "new_refresh_token"
        });

        let mock = server.mock(|when, then| {
            when.method(POST)
                .path("/protocol/openid-connect/token")
                .header("content-type", "application/x-www-form-urlencoded")
                .body_contains("grant_type=refresh_token");

            then.status(200)
                .header("content-type", "application/json")
                .json_body(token_response.clone());
        });

        let mut auth = AuthConfig {
            auth_url: server.url(""),
            client_id: "test-client".into(),
            refresh_token: Some("old_refresh_token".into()),
            secret: None,
        };

        let token_manager = TokenManager::new();
        let credentials_provider = MockCredentialsProvider;

        let token = token_manager
            .get_or_refresh_token(&mut auth, false, &[], &credentials_provider)
            .await
            .unwrap();

        mock.assert();
        assert_eq!(token, "new_access_token");
        assert_eq!(auth.refresh_token, Some("new_refresh_token".into()));
    }

    #[tokio::test]
    async fn test_get_or_refresh_token_with_invalid_refresh_token_fallbacks_to_prompt() {
        let server = MockServer::start();

        let _fail_mock = server.mock(|when, then| {
            when.path("/protocol/openid-connect/token")
                .body_contains("grant_type=refresh_token");
            then.status(400);
        });

        let _success_mock = server.mock(|when, then| {
            when.path("/protocol/openid-connect/token")
                .body_contains("grant_type=password");
            then.status(200).json_body(serde_json::json!({
                "access_token": "new_token",
                "refresh_token": "fresh_token"
            }));
        });

        let mut auth = AuthConfig {
            auth_url: server.url(""),
            client_id: "client_id".into(),
            refresh_token: Some("invalid".into()),
            secret: None,
        };

        let token_manager = TokenManager::new();
        let credentials_provider = MockCredentialsProvider;

        let token = token_manager
            .get_or_refresh_token(&mut auth, false, &[], &credentials_provider)
            .await
            .unwrap();

        assert_eq!(token, "new_token");
        assert_eq!(auth.refresh_token, Some("fresh_token".into()));
    }

    #[tokio::test]
    async fn test_get_or_refresh_token_with_secret() {
        let server = MockServer::start();

        let token_response = serde_json::json!({
            "access_token": "access_with_secret",
            "refresh_token": "refresh_with_secret"
        });

        let mock = server.mock(|when, then| {
            when.method(POST)
                .path("/protocol/openid-connect/token")
                .header("content-type", "application/x-www-form-urlencoded")
                .body_contains("grant_type=password")
                .body_contains("secret=my_secret");

            then.status(200)
                .header("content-type", "application/json")
                .json_body(token_response.clone());
        });

        let mut auth = AuthConfig {
            auth_url: server.url(""),
            client_id: "test-client".into(),
            refresh_token: None,
            secret: Some("my_secret".into()),
        };

        let token_manager = TokenManager::new();
        let credentials_provider = MockCredentialsProvider;

        let token = token_manager
            .get_or_refresh_token(&mut auth, false, &[], &credentials_provider)
            .await
            .unwrap();

        mock.assert();
        assert_eq!(token, "access_with_secret");
        assert_eq!(auth.refresh_token, Some("refresh_with_secret".into()));
    }

    #[tokio::test]
    async fn test_get_or_refresh_token_request_refresh_token_directly() {
        let server = MockServer::start();

        let token_response = serde_json::json!({
            "access_token": "access_token",
            "refresh_token": "existing_refresh_token"
        });

        let mock = server.mock(|when, then| {
            when.method(POST)
                .path("/protocol/openid-connect/token")
                .header("content-type", "application/x-www-form-urlencoded")
                .body_contains("grant_type=refresh_token");

            then.status(200)
                .header("content-type", "application/json")
                .json_body(token_response.clone());
        });

        let mut auth = AuthConfig {
            auth_url: server.url(""),
            client_id: "test-client".into(),
            refresh_token: Some("existing_refresh_token".into()),
            secret: None,
        };

        let token_manager = TokenManager::new();
        let credentials_provider = MockCredentialsProvider;

        // Request refresh token directly instead of access token
        let token = token_manager
            .get_or_refresh_token(&mut auth, true, &[], &credentials_provider)
            .await
            .unwrap();

        mock.assert();
        // Should return the refresh token, not the access token
        assert_eq!(token, "existing_refresh_token");
    }

    #[test]
    fn test_save_and_read_config() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("config.json");

        let mut clients = HashMap::new();
        clients.insert(
            "example".into(),
            AuthConfig {
                auth_url: "https://example.com".into(),
                client_id: "abc123".into(),
                refresh_token: Some("xyz".into()),
                secret: None,
            },
        );

        let config = ConfigFile { clients };
        let config_manager = ConfigManager::new();

        config_manager.save_config(&config_path, &config).unwrap();

        let read_back = config_manager.read_config(&config_path).unwrap();
        assert_eq!(read_back.clients["example"].client_id, "abc123");
    }

    #[test]
    fn test_read_config_missing_file() {
        let path = Path::new("nonexistent_config.json");
        let config_manager = ConfigManager::new();
        let result = config_manager.read_config(path);
        assert!(result.is_none());
    }

    #[test]
    fn test_get_client() {
        let mut clients = HashMap::new();
        clients.insert(
            "example".into(),
            AuthConfig {
                auth_url: "https://example.com".into(),
                client_id: "abc123".into(),
                refresh_token: Some("xyz".into()),
                secret: None,
            },
        );

        let config = ConfigFile { clients };
        let config_manager = ConfigManager::new();

        let client = config_manager.get_client(&config, "example");
        assert!(client.is_some());
        assert_eq!(client.unwrap().client_id, "abc123");

        let missing_client = config_manager.get_client(&config, "missing");
        assert!(missing_client.is_none());
    }
}
