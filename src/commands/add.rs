use super::{CommandContext, CommandHandler};
use crate::types::AuthConfig;
use std::error::Error;

pub struct AddCommand {
    pub nickname: Option<String>,
    pub auth_url: String,
    pub client_id: String,
    pub secret: Option<String>,
}

impl CommandHandler for AddCommand {
    async fn execute(&self, context: CommandContext<'_>) -> Result<(), Box<dyn Error>> {
        let nickname = match &self.nickname {
            Some(name) => name,
            None => &self.client_id,
        };

        let auth_config = AuthConfig {
            auth_url: self.auth_url.clone(),
            client_id: self.client_id.clone(),
            refresh_token: None,
            secret: self.secret.clone(),
        };

        context
            .config_manager
            .add_client(context.config, nickname.to_owned(), auth_config);

        let config_path = context.config_manager.get_config_path();
        context
            .config_manager
            .save_config(&config_path, context.config)?;

        println!("Client '{nickname}' added.");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        commands::{CommandContext, CommandHandler, add::AddCommand},
        config::ConfigManager,
        oauth::TokenManager,
        types::{ConfigFile, CredentialsProvider},
    };

    use tempfile::tempdir;

    struct MockCredentialsProvider;

    impl CredentialsProvider for MockCredentialsProvider {
        fn get_credentials(&self) -> Result<(String, String), Box<dyn std::error::Error>> {
            Ok(("user".into(), "pass".into()))
        }
    }

    #[tokio::test]
    async fn test_add_command() {
        let _dir = tempdir().unwrap();

        let config_manager = ConfigManager::new();
        let token_manager = TokenManager::new();
        let mut config = ConfigFile::default();

        let add_command = AddCommand {
            nickname: Some("test_client".to_string()),
            auth_url: "https://example.com".to_string(),
            client_id: "client123".to_string(),
            secret: None,
        };

        let mock_credentials_provider = MockCredentialsProvider;
        let context = CommandContext {
            config: &mut config,
            config_manager: &config_manager,
            token_manager: &token_manager,
            credentials_provider: &mock_credentials_provider,
        };

        let result = add_command.execute(context).await;
        assert!(result.is_ok());
        assert!(config.clients.contains_key("test_client"));

        let added_client = &config.clients["test_client"];
        assert_eq!(added_client.auth_url, "https://example.com");
        assert_eq!(added_client.client_id, "client123");
    }

    #[tokio::test]
    async fn test_add_command_without_nickname() {
        let config_manager = ConfigManager::new();
        let token_manager = TokenManager::new();
        let mut config = ConfigFile::default();

        let add_command = AddCommand {
            nickname: None,
            auth_url: "https://example.com".to_string(),
            client_id: "client123".to_string(),
            secret: Some("secret".to_string()),
        };

        let mock_credentials_provider = MockCredentialsProvider;
        let context = CommandContext {
            config: &mut config,
            config_manager: &config_manager,
            token_manager: &token_manager,
            credentials_provider: &mock_credentials_provider,
        };

        let result = add_command.execute(context).await;
        assert!(result.is_ok());
        assert!(config.clients.contains_key("client123"));
        assert_eq!(
            config
                .clients
                .get("client123")
                .and_then(|client| client.secret.clone()),
            Some("secret".to_string())
        );
    }
}
