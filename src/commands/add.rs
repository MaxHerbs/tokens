use super::{CommandContext, CommandHandler};
use crate::types::AuthConfig;
use std::error::Error;

pub struct AddCommand {
    pub nickname: Option<String>,
    pub auth_url: String,
    pub client_id: String,
}

impl CommandHandler for AddCommand {
    async fn execute(&self, context: CommandContext<'_>) -> Result<(), Box<dyn Error>> {
        let nickname = self
            .nickname
            .clone()
            .unwrap_or_else(|| self.client_id.clone());

        let auth_config = AuthConfig {
            auth_url: self.auth_url.clone(),
            client_id: self.client_id.clone(),
            refresh_token: None,
        };

        context
            .config_manager
            .add_client(context.config, nickname.clone(), auth_config);

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
        types::ConfigFile,
    };

    use tempfile::tempdir;

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
            secret: Some("secret123".to_string()),
        };

        let context = CommandContext {
            config: &mut config,
            config_manager: &config_manager,
            token_manager: &token_manager,
        };

        let result = add_command.execute(context).await;
        assert!(result.is_ok());
        assert!(config.clients.contains_key("test_client"));

        let added_client = &config.clients["test_client"];
        assert_eq!(added_client.auth_url, "https://example.com");
        assert_eq!(added_client.client_id, "client123");
        assert_eq!(added_client.secret, Some("secret123".to_string()));
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
            secret: None,
        };

        let context = CommandContext {
            config: &mut config,
            config_manager: &config_manager,
            token_manager: &token_manager,
        };

        let result = add_command.execute(context).await;
        assert!(result.is_ok());
        assert!(config.clients.contains_key("client123"));
    }
}
