//! Google Drive OAuth connector implementation.
//!
//! Implements the `RemoteConnector` trait for user-account Google Drive
//! linking via OAuth 2.0 with PKCE. Uses `reqwest` directly for token
//! exchange, refresh, and revocation (see KD4 in session plan).

use async_trait::async_trait;

use super::crypto::{decrypt_credentials, encrypt_credentials};
use super::{ConnectorError, RefreshedToken, RemoteConnector, TokenSet, UserInfo};
use crate::config::GoogleDriveConnectorConfig;

const GOOGLE_AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const GOOGLE_TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const GOOGLE_REVOKE_URL: &str = "https://oauth2.googleapis.com/revoke";
const GOOGLE_USERINFO_URL: &str = "https://www.googleapis.com/oauth2/v2/userinfo";
const DRIVE_SCOPE: &str = "https://www.googleapis.com/auth/drive.readonly openid email profile";
const DEFAULT_REDIRECT_URI: &str = "http://localhost:3001/api/connectors/google-drive/callback";

/// Google Drive OAuth connector.
pub struct GoogleDriveConnector {
    client_id: String,
    client_secret: String,
    redirect_uri: String,
    http_client: reqwest::Client,
}

impl GoogleDriveConnector {
    /// Create a connector from application config.
    ///
    /// Returns `ConnectorError::NotConfigured` if `client_id` or
    /// `client_secret` is missing.
    pub fn new(config: &GoogleDriveConnectorConfig) -> Result<Self, ConnectorError> {
        let client_id = config
            .client_id
            .as_deref()
            .filter(|s| !s.is_empty())
            .ok_or_else(|| ConnectorError::NotConfigured("google_drive client_id not set".into()))?
            .to_string();

        let client_secret = config
            .client_secret
            .as_deref()
            .filter(|s| !s.is_empty())
            .ok_or_else(|| {
                ConnectorError::NotConfigured("google_drive client_secret not set".into())
            })?
            .to_string();

        let redirect_uri = config
            .redirect_uri
            .as_deref()
            .filter(|s| !s.is_empty())
            .unwrap_or(DEFAULT_REDIRECT_URI)
            .to_string();

        Ok(Self {
            client_id,
            client_secret,
            redirect_uri,
            http_client: reqwest::Client::new(),
        })
    }

    /// Build with an explicit HTTP client (for testing with wiremock).
    #[cfg(test)]
    pub fn with_client(
        config: &GoogleDriveConnectorConfig,
        client: reqwest::Client,
    ) -> Result<Self, ConnectorError> {
        let mut connector = Self::new(config)?;
        connector.http_client = client;
        Ok(connector)
    }

    /// Build with an explicit HTTP client and custom URLs (for testing).
    #[cfg(test)]
    pub fn with_test_urls(
        config: &GoogleDriveConnectorConfig,
        client: reqwest::Client,
        token_url: String,
        revoke_url: String,
        userinfo_url: String,
    ) -> Result<Self, ConnectorError> {
        let mut connector = Self::new(config)?;
        connector.http_client = client;
        // Store test URLs in a thread-local for the test instance.
        // We use a different approach: return the connector and let the
        // test methods use the injected URLs through overridden endpoints.
        TEST_URLS.with(|urls| {
            *urls.borrow_mut() = Some(TestUrls {
                token_url,
                revoke_url,
                userinfo_url,
            });
        });
        Ok(connector)
    }

    fn token_url(&self) -> &str {
        #[cfg(test)]
        {
            // Check thread-local override.
            TEST_URLS.with(|urls| {
                if let Some(ref u) = *urls.borrow() {
                    // Leak a &str for the test lifetime. Tests are short-lived.
                    return unsafe { &*(u.token_url.as_str() as *const str) };
                }
                GOOGLE_TOKEN_URL
            })
        }
        #[cfg(not(test))]
        GOOGLE_TOKEN_URL
    }

    fn revoke_url(&self) -> &str {
        #[cfg(test)]
        {
            TEST_URLS.with(|urls| {
                if let Some(ref u) = *urls.borrow() {
                    return unsafe { &*(u.revoke_url.as_str() as *const str) };
                }
                GOOGLE_REVOKE_URL
            })
        }
        #[cfg(not(test))]
        GOOGLE_REVOKE_URL
    }

    fn userinfo_url(&self) -> &str {
        #[cfg(test)]
        {
            TEST_URLS.with(|urls| {
                if let Some(ref u) = *urls.borrow() {
                    return unsafe { &*(u.userinfo_url.as_str() as *const str) };
                }
                GOOGLE_USERINFO_URL
            })
        }
        #[cfg(not(test))]
        GOOGLE_USERINFO_URL
    }
}

#[cfg(test)]
struct TestUrls {
    token_url: String,
    revoke_url: String,
    userinfo_url: String,
}

#[cfg(test)]
thread_local! {
    static TEST_URLS: std::cell::RefCell<Option<TestUrls>> = const { std::cell::RefCell::new(None) };
}

#[async_trait]
impl RemoteConnector for GoogleDriveConnector {
    fn connector_type(&self) -> &str {
        "google_drive"
    }

    fn authorization_url(
        &self,
        state: &str,
        code_challenge: &str,
    ) -> Result<String, ConnectorError> {
        let params = [
            ("response_type", "code"),
            ("client_id", &self.client_id),
            ("redirect_uri", &self.redirect_uri),
            ("scope", DRIVE_SCOPE),
            ("state", state),
            ("code_challenge", code_challenge),
            ("code_challenge_method", "S256"),
            ("access_type", "offline"),
            ("prompt", "consent"),
        ];

        let query = params
            .iter()
            .map(|(k, v)| format!("{}={}", urlencoding(k), urlencoding(v)))
            .collect::<Vec<_>>()
            .join("&");

        Ok(format!("{GOOGLE_AUTH_URL}?{query}"))
    }

    async fn exchange_code(
        &self,
        code: &str,
        code_verifier: &str,
    ) -> Result<TokenSet, ConnectorError> {
        let resp = self
            .http_client
            .post(self.token_url())
            .form(&[
                ("grant_type", "authorization_code"),
                ("code", code),
                ("redirect_uri", &self.redirect_uri),
                ("client_id", &self.client_id),
                ("client_secret", &self.client_secret),
                ("code_verifier", code_verifier),
            ])
            .send()
            .await
            .map_err(|e| ConnectorError::TokenExchange(format!("request failed: {e}")))?;

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(ConnectorError::TokenExchange(format!(
                "token endpoint error: {body}"
            )));
        }

        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| ConnectorError::TokenExchange(format!("invalid response: {e}")))?;

        let access_token = body["access_token"]
            .as_str()
            .ok_or_else(|| ConnectorError::TokenExchange("missing access_token".into()))?
            .to_string();

        let refresh_token = body["refresh_token"]
            .as_str()
            .ok_or_else(|| ConnectorError::TokenExchange("missing refresh_token".into()))?
            .to_string();

        let expires_in_secs = body["expires_in"].as_i64().unwrap_or(3600);
        let scope = body["scope"].as_str().unwrap_or("").to_string();

        Ok(TokenSet {
            access_token,
            refresh_token,
            expires_in_secs,
            scope,
        })
    }

    async fn refresh_access_token(
        &self,
        encrypted_refresh: &[u8],
        key: &[u8],
    ) -> Result<RefreshedToken, ConnectorError> {
        let refresh_token_bytes = decrypt_credentials(encrypted_refresh, key)?;
        let refresh_token = String::from_utf8(refresh_token_bytes)
            .map_err(|e| ConnectorError::Encryption(format!("invalid utf-8: {e}")))?;

        let resp = self
            .http_client
            .post(self.token_url())
            .form(&[
                ("grant_type", "refresh_token"),
                ("refresh_token", &refresh_token),
                ("client_id", &self.client_id),
                ("client_secret", &self.client_secret),
            ])
            .send()
            .await
            .map_err(|e| ConnectorError::TokenRefresh(format!("request failed: {e}")))?;

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(ConnectorError::TokenRefresh(format!(
                "refresh failed: {body}"
            )));
        }

        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| ConnectorError::TokenRefresh(format!("invalid response: {e}")))?;

        let access_token = body["access_token"]
            .as_str()
            .ok_or_else(|| ConnectorError::TokenRefresh("missing access_token".into()))?
            .to_string();

        let expires_in_secs = body["expires_in"].as_i64().unwrap_or(3600);

        Ok(RefreshedToken {
            access_token,
            expires_in_secs,
        })
    }

    async fn revoke(&self, encrypted_refresh: &[u8], key: &[u8]) -> Result<(), ConnectorError> {
        let refresh_token_bytes = decrypt_credentials(encrypted_refresh, key)?;
        let refresh_token = String::from_utf8(refresh_token_bytes)
            .map_err(|e| ConnectorError::Encryption(format!("invalid utf-8: {e}")))?;

        let resp = self
            .http_client
            .post(self.revoke_url())
            .form(&[("token", refresh_token.as_str())])
            .send()
            .await
            .map_err(|e| ConnectorError::Revocation(format!("request failed: {e}")))?;

        // 200 = success, 400 = already revoked (both acceptable).
        if !resp.status().is_success() && resp.status().as_u16() != 400 {
            let body = resp.text().await.unwrap_or_default();
            return Err(ConnectorError::Revocation(format!(
                "revocation failed: {body}"
            )));
        }

        Ok(())
    }

    async fn user_info(&self, access_token: &str) -> Result<UserInfo, ConnectorError> {
        let resp = self
            .http_client
            .get(self.userinfo_url())
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|e| ConnectorError::Network(format!("userinfo request failed: {e}")))?;

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(ConnectorError::Network(format!("userinfo error: {body}")));
        }

        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| ConnectorError::Network(format!("invalid userinfo response: {e}")))?;

        let email = body["email"]
            .as_str()
            .ok_or_else(|| ConnectorError::Network("missing email in userinfo".into()))?
            .to_string();

        let display_name = body["name"].as_str().map(|s| s.to_string());

        Ok(UserInfo {
            email,
            display_name,
        })
    }
}

/// Percent-encode a string for use in a URL query parameter.
fn urlencoding(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for byte in s.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                result.push(byte as char);
            }
            _ => {
                result.push_str(&format!("%{byte:02X}"));
            }
        }
    }
    result
}

/// Encrypt a refresh token for storage.
pub fn encrypt_refresh_token(refresh_token: &str, key: &[u8]) -> Result<Vec<u8>, ConnectorError> {
    encrypt_credentials(refresh_token.as_bytes(), key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn test_config() -> GoogleDriveConnectorConfig {
        GoogleDriveConnectorConfig {
            client_id: Some("test-client-id".into()),
            client_secret: Some("test-client-secret".into()),
            redirect_uri: Some("http://localhost:3001/callback".into()),
        }
    }

    #[test]
    fn authorization_url_contains_required_params() {
        let connector = GoogleDriveConnector::new(&test_config()).unwrap();
        let url = connector
            .authorization_url("test-state", "test-challenge")
            .unwrap();

        assert!(url.starts_with(GOOGLE_AUTH_URL));
        assert!(url.contains("response_type=code"));
        assert!(url.contains("client_id=test-client-id"));
        assert!(url.contains("state=test-state"));
        assert!(url.contains("code_challenge=test-challenge"));
        assert!(url.contains("code_challenge_method=S256"));
        assert!(url.contains("access_type=offline"));
        assert!(url.contains("prompt=consent"));
    }

    #[test]
    fn not_configured_without_client_id() {
        let config = GoogleDriveConnectorConfig {
            client_id: None,
            client_secret: Some("secret".into()),
            redirect_uri: None,
        };
        let result = GoogleDriveConnector::new(&config);
        assert!(matches!(result, Err(ConnectorError::NotConfigured(_))));
    }

    #[test]
    fn not_configured_without_client_secret() {
        let config = GoogleDriveConnectorConfig {
            client_id: Some("id".into()),
            client_secret: None,
            redirect_uri: None,
        };
        let result = GoogleDriveConnector::new(&config);
        assert!(matches!(result, Err(ConnectorError::NotConfigured(_))));
    }

    #[tokio::test]
    async fn exchange_code_happy_path() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "access_token": "ya29.test-access",
                "refresh_token": "1//test-refresh",
                "expires_in": 3600,
                "scope": "drive.readonly email profile openid",
                "token_type": "Bearer"
            })))
            .mount(&mock_server)
            .await;

        let config = test_config();
        let connector = GoogleDriveConnector::with_test_urls(
            &config,
            reqwest::Client::new(),
            format!("{}/token", mock_server.uri()),
            format!("{}/revoke", mock_server.uri()),
            format!("{}/userinfo", mock_server.uri()),
        )
        .unwrap();

        let tokens = connector
            .exchange_code("auth-code", "verifier")
            .await
            .unwrap();
        assert_eq!(tokens.access_token, "ya29.test-access");
        assert_eq!(tokens.refresh_token, "1//test-refresh");
        assert_eq!(tokens.expires_in_secs, 3600);
    }

    #[tokio::test]
    async fn exchange_code_invalid_grant() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/token"))
            .respond_with(ResponseTemplate::new(400).set_body_json(serde_json::json!({
                "error": "invalid_grant",
                "error_description": "Code expired"
            })))
            .mount(&mock_server)
            .await;

        let config = test_config();
        let connector = GoogleDriveConnector::with_test_urls(
            &config,
            reqwest::Client::new(),
            format!("{}/token", mock_server.uri()),
            format!("{}/revoke", mock_server.uri()),
            format!("{}/userinfo", mock_server.uri()),
        )
        .unwrap();

        let result = connector.exchange_code("bad-code", "verifier").await;
        assert!(matches!(result, Err(ConnectorError::TokenExchange(_))));
    }

    #[tokio::test]
    async fn refresh_access_token_happy_path() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "access_token": "ya29.refreshed",
                "expires_in": 3600,
                "token_type": "Bearer"
            })))
            .mount(&mock_server)
            .await;

        let key: Vec<u8> = (0..32).collect();
        let encrypted = encrypt_refresh_token("1//refresh-token", &key).unwrap();

        let config = test_config();
        let connector = GoogleDriveConnector::with_test_urls(
            &config,
            reqwest::Client::new(),
            format!("{}/token", mock_server.uri()),
            format!("{}/revoke", mock_server.uri()),
            format!("{}/userinfo", mock_server.uri()),
        )
        .unwrap();

        let refreshed = connector
            .refresh_access_token(&encrypted, &key)
            .await
            .unwrap();
        assert_eq!(refreshed.access_token, "ya29.refreshed");
    }

    #[tokio::test]
    async fn refresh_access_token_revoked() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/token"))
            .respond_with(ResponseTemplate::new(401).set_body_json(serde_json::json!({
                "error": "invalid_grant",
                "error_description": "Token has been revoked"
            })))
            .mount(&mock_server)
            .await;

        let key: Vec<u8> = (0..32).collect();
        let encrypted = encrypt_refresh_token("1//revoked-token", &key).unwrap();

        let config = test_config();
        let connector = GoogleDriveConnector::with_test_urls(
            &config,
            reqwest::Client::new(),
            format!("{}/token", mock_server.uri()),
            format!("{}/revoke", mock_server.uri()),
            format!("{}/userinfo", mock_server.uri()),
        )
        .unwrap();

        let result = connector.refresh_access_token(&encrypted, &key).await;
        assert!(matches!(result, Err(ConnectorError::TokenRefresh(_))));
    }

    #[tokio::test]
    async fn revoke_happy_path() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/revoke"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock_server)
            .await;

        let key: Vec<u8> = (0..32).collect();
        let encrypted = encrypt_refresh_token("1//to-revoke", &key).unwrap();

        let config = test_config();
        let connector = GoogleDriveConnector::with_test_urls(
            &config,
            reqwest::Client::new(),
            format!("{}/token", mock_server.uri()),
            format!("{}/revoke", mock_server.uri()),
            format!("{}/userinfo", mock_server.uri()),
        )
        .unwrap();

        connector.revoke(&encrypted, &key).await.unwrap();
    }

    #[tokio::test]
    async fn revoke_already_revoked_succeeds() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/revoke"))
            .respond_with(ResponseTemplate::new(400).set_body_json(serde_json::json!({
                "error": "invalid_token"
            })))
            .mount(&mock_server)
            .await;

        let key: Vec<u8> = (0..32).collect();
        let encrypted = encrypt_refresh_token("1//already-revoked", &key).unwrap();

        let config = test_config();
        let connector = GoogleDriveConnector::with_test_urls(
            &config,
            reqwest::Client::new(),
            format!("{}/token", mock_server.uri()),
            format!("{}/revoke", mock_server.uri()),
            format!("{}/userinfo", mock_server.uri()),
        )
        .unwrap();

        // Should succeed even with 400 (already revoked).
        connector.revoke(&encrypted, &key).await.unwrap();
    }

    #[tokio::test]
    async fn user_info_parses_email_and_name() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/userinfo"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "email": "user@example.com",
                "name": "Test User",
                "picture": "https://example.com/photo.jpg"
            })))
            .mount(&mock_server)
            .await;

        let config = test_config();
        let connector = GoogleDriveConnector::with_test_urls(
            &config,
            reqwest::Client::new(),
            format!("{}/token", mock_server.uri()),
            format!("{}/revoke", mock_server.uri()),
            format!("{}/userinfo", mock_server.uri()),
        )
        .unwrap();

        let info = connector.user_info("test-access-token").await.unwrap();
        assert_eq!(info.email, "user@example.com");
        assert_eq!(info.display_name.as_deref(), Some("Test User"));
    }
}
