use tokens::tokens::*;

#[test]
fn test_list_clients() {
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

    let table = list_clients(&mut config);
    let table_string = table.to_string();

    assert!(table_string.contains("Nickname"));
    assert!(table_string.contains("ClientId"));
    assert!(table_string.contains("test1"));
    assert!(table_string.contains("client_id_1"));
    assert!(table_string.contains("test2"));
    assert!(table_string.contains("client_id_2"));
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

    delete_client("test1", &mut config);

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
