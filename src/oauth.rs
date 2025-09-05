use crate::types::{AuthConfig, CredentialsProvider, TokenResponse};
use reqwest::Client;
use std::error::Error;

#[derive(Default)]
pub struct TokenManager {
    client: Client,
}

impl TokenManager {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    pub async fn get_or_refresh_token(
        &self,
        auth: &mut AuthConfig,
        fetch_refresh_token: bool,
        scopes: &[String],
        credentials_provider: &dyn CredentialsProvider,
    ) -> Result<String, Box<dyn Error>> {
        if let Some(ref refresh_token) = auth.refresh_token.clone()
            && let Ok(token) = self.use_refresh_token(auth, refresh_token, scopes).await
        {
            return if fetch_refresh_token {
                Ok(refresh_token.clone())
            } else {
                Ok(token)
            };
        }

        let (username, password) = credentials_provider.get_credentials()?;
        self.request_new_token(auth, &username, &password, scopes)
            .await
    }

    async fn request_new_token(
        &self,
        auth: &mut AuthConfig,
        username: &str,
        password: &str,
        scopes: &[String],
    ) -> Result<String, Box<dyn Error>> {
        let mut form = vec![
            ("grant_type", "password"),
            ("client_id", &auth.client_id),
            ("username", username),
            ("password", password),
        ];

        self.add_optional_fields(&mut form, auth, scopes);

        let url = format!("{}/protocol/openid-connect/token", auth.auth_url);
        let res = self
            .client
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

    async fn use_refresh_token(
        &self,
        auth: &mut AuthConfig,
        refresh_token: &str,
        scopes: &[String],
    ) -> Result<String, reqwest::Error> {
        let mut form = vec![
            ("grant_type", "refresh_token"),
            ("client_id", &auth.client_id),
            ("refresh_token", refresh_token),
        ];

        self.add_optional_fields(&mut form, auth, scopes);

        let url = format!("{}/protocol/openid-connect/token", auth.auth_url);

        let res = self
            .client
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

    fn add_optional_fields<'a>(
        &self,
        form: &mut Vec<(&str, &'a str)>,
        _auth: &'a AuthConfig,
        scopes: &[String],
    ) {
        if !scopes.is_empty() {
            let scopes_str: &'a str = Box::leak(scopes.join(" ").into_boxed_str());
            form.push(("scope", scopes_str));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::AuthConfig;
    use mockito::{Matcher::Regex, Server};

    struct MockCredentialsProvider;

    impl CredentialsProvider for MockCredentialsProvider {
        fn get_credentials(&self) -> Result<(String, String), Box<dyn Error>> {
            Ok(("user".into(), "pass".into()))
        }
    }

    #[tokio::test]
    async fn ensure_scopes() {
        let mock_response = r#"{"access_token": "token", "refresh_token": "refresh"}"#;
        let mut server = Server::new_async().await;

        let mock = server
            .mock("POST", "/realms/master/protocol/openid-connect/token")
            .with_status(200)
            .match_body(Regex("scope=openid\\+profile".into()))
            .with_header("content-type", "application/json")
            .with_body(mock_response)
            .create_async()
            .await;

        let mut auth = AuthConfig {
            auth_url: format!("{}/realms/master", server.url()),
            client_id: "test".to_string(),
            refresh_token: None,
        };

        let token_manager = TokenManager::new();
        let credentials_provider = MockCredentialsProvider;
        let scopes = vec!["openid".to_string(), "profile".to_string()];

        let token = token_manager
            .get_or_refresh_token(&mut auth, false, &scopes, &credentials_provider)
            .await
            .unwrap();

        assert_eq!(token, "token");
        mock.assert_async().await;
    }
}
