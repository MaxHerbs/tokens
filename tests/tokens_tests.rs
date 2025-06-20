use httpmock::Method::POST;
use httpmock::MockServer;
use std::collections::HashMap;
use std::path::Path;
use tempfile::tempdir;
use tokens::tokens::*;

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
    };

    let token = get_or_refresh_token_with_input(&mut auth, false, || {
        panic!("Should not prompt credentials if refresh works");
    })
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
    };

    let token =
        get_or_refresh_token_with_input(&mut auth, false, || Ok(("user".into(), "pass".into())))
            .await
            .unwrap();

    assert_eq!(token, "new_token");
    assert_eq!(auth.refresh_token, Some("fresh_token".into()));
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
        },
    );

    let config = ConfigFile { clients };

    save_config(&config_path, &config).unwrap();

    let read_back = read_config(&config_path).unwrap();
    assert_eq!(read_back.clients["example"].client_id, "abc123");
}

#[test]
fn test_read_config_missing_file() {
    let path = Path::new("nonexistent_config.json");
    let result = read_config(path);
    assert!(result.is_none());
}
