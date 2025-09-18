use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct AuthConfig {
    pub auth_url: String,
    pub client_id: String,
    pub refresh_token: Option<String>,
    pub secret: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default, PartialEq)]
pub struct ConfigFile {
    pub clients: HashMap<String, AuthConfig>,
}

#[derive(Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: Option<String>,
}

pub trait CredentialsProvider {
    fn get_credentials(&self) -> Result<(String, String), Box<dyn std::error::Error>>;
}

pub struct ConsoleCredentialsProvider;

impl CredentialsProvider for ConsoleCredentialsProvider {
    fn get_credentials(&self) -> Result<(String, String), Box<dyn std::error::Error>> {
        use rpassword::read_password;
        use std::io::{self, Write};

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
}

#[cfg(test)]
mod tests {

    use std::collections::HashMap;

    use crate::types::{AuthConfig, ConfigFile, CredentialsProvider, TokenResponse};

    #[test]
    fn test_auth_config_serialization() {
        let auth_config = AuthConfig {
            auth_url: "https://example.com".to_string(),
            client_id: "client123".to_string(),
            refresh_token: Some("refresh123".to_string()),
            secret: None,
        };

        let serialized = serde_json::to_string(&auth_config).unwrap();
        let deserialized: AuthConfig = serde_json::from_str(&serialized).unwrap();

        assert_eq!(auth_config, deserialized);
    }

    #[test]
    fn test_auth_config_serialization_with_none_values() {
        let auth_config = AuthConfig {
            auth_url: "https://example.com".to_string(),
            client_id: "client123".to_string(),
            refresh_token: None,
            secret: None,
        };

        let serialized = serde_json::to_string(&auth_config).unwrap();
        let deserialized: AuthConfig = serde_json::from_str(&serialized).unwrap();

        assert_eq!(auth_config, deserialized);
    }

    #[test]
    fn test_config_file_default() {
        let config = ConfigFile::default();
        assert!(config.clients.is_empty());
    }

    #[test]
    fn test_config_file_serialization() {
        let mut clients = HashMap::new();
        clients.insert(
            "test_client".to_string(),
            AuthConfig {
                auth_url: "https://example.com".to_string(),
                client_id: "client123".to_string(),
                refresh_token: Some("refresh123".to_string()),
                secret: None,
            },
        );

        let config_file = ConfigFile { clients };

        let serialized = serde_json::to_string(&config_file).unwrap();
        let deserialized: ConfigFile = serde_json::from_str(&serialized).unwrap();

        assert_eq!(config_file, deserialized);
    }

    #[test]
    fn test_token_response_deserialization_with_refresh_token() {
        let json = r#"{"access_token": "access123", "refresh_token": "refresh123"}"#;
        let token_response: TokenResponse = serde_json::from_str(json).unwrap();

        assert_eq!(token_response.access_token, "access123");
        assert_eq!(token_response.refresh_token, Some("refresh123".to_string()));
    }

    #[test]
    fn test_token_response_deserialization_without_refresh_token() {
        let json = r#"{"access_token": "access123"}"#;
        let token_response: TokenResponse = serde_json::from_str(json).unwrap();

        assert_eq!(token_response.access_token, "access123");
        assert_eq!(token_response.refresh_token, None);
    }

    struct TestCredentialsProvider {
        username: String,
        password: String,
    }

    impl CredentialsProvider for TestCredentialsProvider {
        fn get_credentials(&self) -> Result<(String, String), Box<dyn std::error::Error>> {
            Ok((self.username.clone(), self.password.clone()))
        }
    }

    #[test]
    fn test_credentials_provider_trait() {
        let provider = TestCredentialsProvider {
            username: "testuser".to_string(),
            password: "testpass".to_string(),
        };

        let result = provider.get_credentials().unwrap();
        assert_eq!(result.0, "testuser");
        assert_eq!(result.1, "testpass");
    }

    struct FailingCredentialsProvider;

    impl CredentialsProvider for FailingCredentialsProvider {
        fn get_credentials(&self) -> Result<(String, String), Box<dyn std::error::Error>> {
            Err("Test error".into())
        }
    }

    #[test]
    fn test_credentials_provider_error_handling() {
        let provider = FailingCredentialsProvider;
        let result = provider.get_credentials();
        assert!(result.is_err());
    }
}
