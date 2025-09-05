use super::{CommandContext, CommandHandler};
use prettytable::{Table, row};
use std::error::Error;

pub struct ListCommand;

impl CommandHandler for ListCommand {
    async fn execute(&self, context: CommandContext<'_>) -> Result<(), Box<dyn Error>> {
        let mut table = Table::new();
        table.add_row(row!["Nickname", "ClientId", "URL"]);

        for (nickname, config) in context.config_manager.list_clients(context.config) {
            table.add_row(row![nickname, &config.client_id, &config.auth_url]);
        }

        table.printstd();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use prettytable::Table;

    use crate::{
        commands::{CommandContext, CommandHandler, list::ListCommand},
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
    fn test_list_clients() {
        let config = ConfigFile {
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
        let clients = config_manager.list_clients(&config);

        let mut table = Table::new();
        table.add_row(prettytable::row!["Nickname", "ClientId", "URL"]);
        for (nickname, config) in clients {
            table.add_row(prettytable::row![
                nickname,
                &config.client_id,
                &config.auth_url
            ]);
        }

        let table_string = table.to_string();

        assert!(table_string.contains("Nickname"));
        assert!(table_string.contains("ClientId"));
        assert!(table_string.contains("test1"));
        assert!(table_string.contains("client_id_1"));
        assert!(table_string.contains("test2"));
        assert!(table_string.contains("client_id_2"));
        assert!(table_string.contains("https://auth2.com"));
    }

    #[tokio::test]
    async fn test_list_command_empty() {
        let config_manager = ConfigManager::new();
        let token_manager = TokenManager::new();
        let mut config = ConfigFile::default();

        let list_command = ListCommand;

        let mock_credentials_provider = MockCredentialsProvider;
        let context = CommandContext {
            config: &mut config,
            config_manager: &config_manager,
            token_manager: &token_manager,
            credentials_provider: &mock_credentials_provider,
        };

        let result = list_command.execute(context).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_list_command_with_clients() {
        let config_manager = ConfigManager::new();
        let token_manager = TokenManager::new();
        let mut config = ConfigFile {
            clients: {
                let mut clients = HashMap::new();
                clients.insert(
                    "client1".to_string(),
                    AuthConfig {
                        auth_url: "https://auth1.com".to_string(),
                        client_id: "id1".to_string(),
                        refresh_token: Some("token1".to_string()),
                    },
                );
                clients.insert(
                    "client2".to_string(),
                    AuthConfig {
                        auth_url: "https://auth2.com".to_string(),
                        client_id: "id2".to_string(),
                        refresh_token: None,
                    },
                );
                clients
            },
        };

        let list_command = ListCommand;

        let mock_credentials_provider = MockCredentialsProvider;
        let context = CommandContext {
            config: &mut config,
            config_manager: &config_manager,
            token_manager: &token_manager,
            credentials_provider: &mock_credentials_provider,
        };

        let result = list_command.execute(context).await;
        assert!(result.is_ok());
    }
}
