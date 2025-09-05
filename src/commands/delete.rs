use super::{CommandContext, CommandHandler};
use std::error::Error;

pub struct DeleteCommand {
    pub nickname: String,
}

impl CommandHandler for DeleteCommand {
    async fn execute(&self, context: CommandContext<'_>) -> Result<(), Box<dyn Error>> {
        if context
            .config_manager
            .remove_client(context.config, &self.nickname)
        {
            let config_path = context.config_manager.get_config_path();
            match context
                .config_manager
                .save_config(&config_path, context.config)
            {
                Ok(()) => println!("Client '{}' removed", self.nickname),
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
        commands::{CommandContext, CommandHandler, delete::DeleteCommand},
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
    fn test_delete_client() {
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
        config_manager.remove_client(&mut config, "test1");

        let target = ConfigFile {
            clients: [(
                "test2".to_string(),
                AuthConfig {
                    auth_url: "https://auth2.com".to_string(),
                    client_id: "client_id_2".to_string(),
                    refresh_token: None,
                },
            )]
            .into_iter()
            .collect(),
        };

        assert_eq!(config, target);
    }

    #[tokio::test]
    async fn test_delete_command_existing_client() {
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

        let delete_command = DeleteCommand {
            nickname: "test_client".to_string(),
        };

        let mock_credentials_provider = MockCredentialsProvider;
        let context = CommandContext {
            config: &mut config,
            config_manager: &config_manager,
            token_manager: &token_manager,
            credentials_provider: &mock_credentials_provider,
        };

        let result = delete_command.execute(context).await;
        assert!(result.is_ok());
        assert!(!config.clients.contains_key("test_client"));
    }

    #[tokio::test]
    async fn test_delete_command_nonexistent_client() {
        let config_manager = ConfigManager::new();
        let token_manager = TokenManager::new();
        let mut config = ConfigFile::default();

        let delete_command = DeleteCommand {
            nickname: "nonexistent_client".to_string(),
        };

        let mock_credentials_provider = MockCredentialsProvider;
        let context = CommandContext {
            config: &mut config,
            config_manager: &config_manager,
            token_manager: &token_manager,
            credentials_provider: &mock_credentials_provider,
        };

        let result = delete_command.execute(context).await;
        assert!(result.is_ok()); // Command succeeds but prints message that client doesn't exist
    }
}
