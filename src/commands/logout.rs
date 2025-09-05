use super::{CommandContext, CommandHandler};
use std::error::Error;

pub struct LogoutCommand {
    pub nickname: String,
}

impl CommandHandler for LogoutCommand {
    async fn execute(&self, context: CommandContext<'_>) -> Result<(), Box<dyn Error>> {
        if let Some(client) = context
            .config_manager
            .get_client_mut(context.config, &self.nickname)
        {
            client.refresh_token = None;
            let config_path = context.config_manager.get_config_path();
            match context
                .config_manager
                .save_config(&config_path, context.config)
            {
                Ok(()) => println!("Refresh token for '{}' removed", self.nickname),
                Err(e) => println!("Failed to update config file.\n{e}"),
            }
        } else {
            println!("Client '{}' doesn't exist", self.nickname);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{
        commands::{CommandContext, CommandHandler, logout::LogoutCommand},
        config::ConfigManager,
        oauth::TokenManager,
        types::{AuthConfig, ConfigFile, CredentialsProvider},
    };

    struct MockCredentialsProvider;

    impl CredentialsProvider for MockCredentialsProvider {
        fn get_credentials(&self) -> Result<(String, String), Box<dyn std::error::Error>> {
            Ok(("user".into(), "pass".into()))
        }
    }

    #[test]
    fn test_logout_client() {
        let mut config = ConfigFile {
            clients: [
                (
                    "test1".to_string(),
                    AuthConfig {
                        auth_url: "https://auth1.com".to_string(),
                        client_id: "client_id_1".to_string(),
                        refresh_token: Some("token1".to_string()),
                    },
                ),
                (
                    "test2".to_string(),
                    AuthConfig {
                        auth_url: "https://auth2.com".to_string(),
                        client_id: "client_id_2".to_string(),
                        refresh_token: None,
                    },
                ),
            ]
            .into_iter()
            .collect(),
        };

        let config_manager = ConfigManager::new();
        if let Some(client) = config_manager.get_client_mut(&mut config, "test1") {
            client.refresh_token = None;
        }

        let target = ConfigFile {
            clients: [
                (
                    "test1".to_string(),
                    AuthConfig {
                        auth_url: "https://auth1.com".to_string(),
                        client_id: "client_id_1".to_string(),
                        refresh_token: None,
                    },
                ),
                (
                    "test2".to_string(),
                    AuthConfig {
                        auth_url: "https://auth2.com".to_string(),
                        client_id: "client_id_2".to_string(),
                        refresh_token: None,
                    },
                ),
            ]
            .into_iter()
            .collect(),
        };

        assert_eq!(config, target);
    }

    #[tokio::test]
    async fn test_logout_command_existing_client() {
        let config_manager = ConfigManager::new();
        let token_manager = TokenManager::new();
        let mut config = ConfigFile {
            clients: {
                let mut clients = HashMap::new();
                clients.insert(
                    "test_client".to_string(),
                    AuthConfig {
                        auth_url: "https://example.com".to_string(),
                        client_id: "client123".to_string(),
                        refresh_token: Some("refresh123".to_string()),
                    },
                );
                clients
            },
        };

        let logout_command = LogoutCommand {
            nickname: "test_client".to_string(),
        };

        let mock_credentials_provider = MockCredentialsProvider;
        let context = CommandContext {
            config: &mut config,
            config_manager: &config_manager,
            token_manager: &token_manager,
            credentials_provider: &mock_credentials_provider,
        };

        let result = logout_command.execute(context).await;
        assert!(result.is_ok());

        // Verify refresh token was cleared
        let client = config.clients.get("test_client").unwrap();
        assert_eq!(client.refresh_token, None);
    }

    #[tokio::test]
    async fn test_logout_command_nonexistent_client() {
        let config_manager = ConfigManager::new();
        let token_manager = TokenManager::new();
        let mut config = ConfigFile::default();

        let logout_command = LogoutCommand {
            nickname: "nonexistent_client".to_string(),
        };

        let mock_credentials_provider = MockCredentialsProvider;
        let context = CommandContext {
            config: &mut config,
            config_manager: &config_manager,
            token_manager: &token_manager,
            credentials_provider: &mock_credentials_provider,
        };

        let result = logout_command.execute(context).await;
        assert!(result.is_ok());
    }
}
