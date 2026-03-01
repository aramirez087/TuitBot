//! Google Drive content source provider.
//!
//! Polls a Google Drive folder for `.md` and `.txt` files using the
//! Drive API v3. Supports two authentication strategies:
//!
//! - **ServiceAccount** (legacy): reads a service-account JSON key file,
//!   builds a JWT, and exchanges it for an access token.
//! - **LinkedAccount** (new): uses encrypted OAuth refresh tokens stored
//!   in the `connections` table, refreshing via the connector module.
//!
//! `connection_id` takes precedence when both are configured.

mod jwt;

use std::path::Path;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use async_trait::async_trait;

use super::{ContentSourceProvider, SourceError, SourceFile};
use crate::automation::watchtower::matches_patterns;
use crate::source::connector::google_drive::GoogleDriveConnector;
use crate::source::connector::{ConnectorError, RemoteConnector};
use crate::storage::DbPool;

// ---------------------------------------------------------------------------
// Auth strategy
// ---------------------------------------------------------------------------

/// How the provider obtains access tokens for Drive API calls.
pub enum DriveAuthStrategy {
    /// Legacy: service-account JSON key file.
    ServiceAccount { key_path: String },
    /// New: linked-account credentials from connections table.
    LinkedAccount {
        connection_id: i64,
        pool: DbPool,
        connector_key: Vec<u8>,
        connector: GoogleDriveConnector,
    },
}

// ---------------------------------------------------------------------------
// Provider
// ---------------------------------------------------------------------------

/// Google Drive content source provider.
///
/// Instantiated when a `google_drive` source is configured with a
/// valid `folder_id` and either a `service_account_key` path or a
/// `connection_id` referencing a linked account.
pub struct GoogleDriveProvider {
    folder_id: String,
    auth_strategy: DriveAuthStrategy,
    http_client: reqwest::Client,
    token_cache: Mutex<Option<CachedToken>>,
}

struct CachedToken {
    access_token: String,
    expires_at: Instant,
}

impl GoogleDriveProvider {
    /// Create a provider using a service-account key file (legacy path).
    pub fn new(folder_id: String, service_account_key_path: String) -> Self {
        Self {
            folder_id,
            auth_strategy: DriveAuthStrategy::ServiceAccount {
                key_path: service_account_key_path,
            },
            http_client: reqwest::Client::new(),
            token_cache: Mutex::new(None),
        }
    }

    /// Create a provider using linked-account OAuth credentials.
    pub fn from_connection(
        folder_id: String,
        connection_id: i64,
        pool: DbPool,
        connector_key: Vec<u8>,
        connector: GoogleDriveConnector,
    ) -> Self {
        Self {
            folder_id,
            auth_strategy: DriveAuthStrategy::LinkedAccount {
                connection_id,
                pool,
                connector_key,
                connector,
            },
            http_client: reqwest::Client::new(),
            token_cache: Mutex::new(None),
        }
    }

    /// Build with an explicit HTTP client (for testing with wiremock).
    #[cfg(test)]
    pub fn with_client(
        folder_id: String,
        service_account_key_path: String,
        client: reqwest::Client,
    ) -> Self {
        Self {
            folder_id,
            auth_strategy: DriveAuthStrategy::ServiceAccount {
                key_path: service_account_key_path,
            },
            http_client: client,
            token_cache: Mutex::new(None),
        }
    }

    /// Build with an explicit HTTP client and linked-account strategy (for testing).
    #[cfg(test)]
    pub fn with_client_and_connection(
        folder_id: String,
        connection_id: i64,
        pool: DbPool,
        connector_key: Vec<u8>,
        connector: GoogleDriveConnector,
        client: reqwest::Client,
    ) -> Self {
        Self {
            folder_id,
            auth_strategy: DriveAuthStrategy::LinkedAccount {
                connection_id,
                pool,
                connector_key,
                connector,
            },
            http_client: client,
            token_cache: Mutex::new(None),
        }
    }

    /// Obtain a valid access token, refreshing if expired.
    async fn get_access_token(&self) -> Result<String, SourceError> {
        // Check cache.
        if let Ok(cache) = self.token_cache.lock() {
            if let Some(ref tok) = *cache {
                if tok.expires_at > Instant::now() + Duration::from_secs(60) {
                    return Ok(tok.access_token.clone());
                }
            }
        }

        let token = match &self.auth_strategy {
            DriveAuthStrategy::ServiceAccount { key_path } => {
                self.fetch_service_account_token(key_path).await?
            }
            DriveAuthStrategy::LinkedAccount {
                connection_id,
                pool,
                connector_key,
                connector,
            } => {
                self.refresh_from_connection(*connection_id, pool, connector_key, connector)
                    .await?
            }
        };

        let access_token = token.access_token.clone();

        if let Ok(mut cache) = self.token_cache.lock() {
            *cache = Some(token);
        }

        Ok(access_token)
    }

    /// Read the service-account key, build a JWT, and exchange for an
    /// access token via Google's token endpoint.
    async fn fetch_service_account_token(
        &self,
        key_path: &str,
    ) -> Result<CachedToken, SourceError> {
        let key_bytes = tokio::fs::read_to_string(key_path).await.map_err(|e| {
            SourceError::Auth(format!("cannot read service account key {key_path}: {e}"))
        })?;

        let key_json: serde_json::Value = serde_json::from_str(&key_bytes)
            .map_err(|e| SourceError::Auth(format!("invalid service account JSON: {e}")))?;

        let client_email = key_json["client_email"]
            .as_str()
            .ok_or_else(|| SourceError::Auth("missing client_email in key".into()))?;

        let private_key_pem = key_json["private_key"]
            .as_str()
            .ok_or_else(|| SourceError::Auth("missing private_key in key".into()))?;

        let token_uri = key_json["token_uri"]
            .as_str()
            .unwrap_or("https://oauth2.googleapis.com/token");

        let now = chrono::Utc::now().timestamp();
        let claims = serde_json::json!({
            "iss": client_email,
            "scope": "https://www.googleapis.com/auth/drive.readonly",
            "aud": token_uri,
            "iat": now,
            "exp": now + 3600,
        });

        let jwt_token = jwt::build_jwt(&claims, private_key_pem)?;

        let resp = self
            .http_client
            .post(token_uri)
            .form(&[
                ("grant_type", "urn:ietf:params:oauth:grant-type:jwt-bearer"),
                ("assertion", &jwt_token),
            ])
            .send()
            .await
            .map_err(|e| SourceError::Auth(format!("token exchange failed: {e}")))?;

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(SourceError::Auth(format!(
                "token endpoint returned error: {body}"
            )));
        }

        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| SourceError::Auth(format!("invalid token response: {e}")))?;

        let access_token = body["access_token"]
            .as_str()
            .ok_or_else(|| SourceError::Auth("no access_token in response".into()))?
            .to_string();

        let expires_in = body["expires_in"].as_u64().unwrap_or(3600);

        Ok(CachedToken {
            access_token,
            expires_at: Instant::now() + Duration::from_secs(expires_in),
        })
    }

    /// Refresh an access token from linked-account credentials.
    ///
    /// 1. Check in-memory cache -- return if valid (>60s remaining).
    /// 2. Read encrypted credentials from DB -- if None, return ConnectionBroken.
    /// 3. Call connector's refresh_access_token.
    /// 4. On success: cache the new token, return it.
    /// 5. On revocation/irrecoverable error: return ConnectionBroken.
    /// 6. On transient error: return Auth error.
    async fn refresh_from_connection(
        &self,
        connection_id: i64,
        pool: &DbPool,
        connector_key: &[u8],
        connector: &GoogleDriveConnector,
    ) -> Result<CachedToken, SourceError> {
        // Read encrypted credentials from DB.
        let encrypted = crate::storage::watchtower::read_encrypted_credentials(pool, connection_id)
            .await
            .map_err(|e| SourceError::ConnectionBroken {
                connection_id,
                reason: format!("failed to read credentials: {e}"),
            })?;

        let encrypted = match encrypted {
            Some(enc) => enc,
            None => {
                return Err(SourceError::ConnectionBroken {
                    connection_id,
                    reason: "no credentials found for connection".into(),
                });
            }
        };

        // Call connector refresh.
        match connector
            .refresh_access_token(&encrypted, connector_key)
            .await
        {
            Ok(refreshed) => {
                let expires_in = refreshed.expires_in_secs.max(0) as u64;
                Ok(CachedToken {
                    access_token: refreshed.access_token,
                    expires_at: Instant::now() + Duration::from_secs(expires_in),
                })
            }
            Err(ConnectorError::TokenRefresh(msg)) if is_revocation_error(&msg) => {
                Err(SourceError::ConnectionBroken {
                    connection_id,
                    reason: format!("token revoked: {msg}"),
                })
            }
            Err(ConnectorError::Encryption(msg)) => Err(SourceError::ConnectionBroken {
                connection_id,
                reason: format!("credential decryption failed: {msg}"),
            }),
            Err(e) => Err(SourceError::Auth(format!(
                "token refresh failed for connection {connection_id}: {e}"
            ))),
        }
    }
}

/// Check if a refresh failure message indicates token revocation.
fn is_revocation_error(msg: &str) -> bool {
    let lower = msg.to_lowercase();
    lower.contains("revoked")
        || lower.contains("invalid_grant")
        || lower.contains("token has been expired or revoked")
}

#[async_trait]
impl ContentSourceProvider for GoogleDriveProvider {
    fn source_type(&self) -> &str {
        "google_drive"
    }

    async fn scan_for_changes(
        &self,
        since_cursor: Option<&str>,
        patterns: &[String],
    ) -> Result<Vec<SourceFile>, SourceError> {
        let token = self.get_access_token().await?;

        let mut q = format!("'{}' in parents and trashed = false", self.folder_id);

        if let Some(cursor) = since_cursor {
            q.push_str(&format!(" and modifiedTime > '{cursor}'"));
        }

        let resp = self
            .http_client
            .get("https://www.googleapis.com/drive/v3/files")
            .bearer_auth(&token)
            .query(&[
                ("q", q.as_str()),
                ("fields", "files(id,name,md5Checksum,modifiedTime,mimeType)"),
                ("pageSize", "1000"),
            ])
            .send()
            .await
            .map_err(|e| SourceError::Network(format!("Drive list failed: {e}")))?;

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(SourceError::Network(format!("Drive API error: {body}")));
        }

        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| SourceError::Network(format!("invalid Drive response: {e}")))?;

        let files = body["files"].as_array().cloned().unwrap_or_default();

        let mut result = Vec::new();
        for file in &files {
            let id = match file["id"].as_str() {
                Some(id) => id,
                None => continue,
            };
            let name = file["name"].as_str().unwrap_or("unknown");

            if !patterns.is_empty() && !matches_patterns(Path::new(name), patterns) {
                continue;
            }

            let hash = file["md5Checksum"].as_str().unwrap_or("").to_string();
            let modified = file["modifiedTime"].as_str().unwrap_or("").to_string();

            result.push(SourceFile {
                provider_id: format!("gdrive://{id}/{name}"),
                display_name: name.to_string(),
                content_hash: hash,
                modified_at: modified,
            });
        }

        Ok(result)
    }

    async fn read_content(&self, file_id: &str) -> Result<String, SourceError> {
        let drive_id = extract_drive_id(file_id)?;
        let token = self.get_access_token().await?;

        let url = format!("https://www.googleapis.com/drive/v3/files/{drive_id}?alt=media");

        let resp = self
            .http_client
            .get(&url)
            .bearer_auth(&token)
            .send()
            .await
            .map_err(|e| SourceError::Network(format!("Drive get failed: {e}")))?;

        if resp.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(SourceError::NotFound(format!("file {drive_id} not found")));
        }

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(SourceError::Network(format!(
                "Drive download error: {body}"
            )));
        }

        resp.text()
            .await
            .map_err(|e| SourceError::Network(format!("read body failed: {e}")))
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Test-only accessor for `extract_drive_id`.
#[cfg(test)]
impl GoogleDriveProvider {
    pub fn extract_drive_id_for_test(provider_id: &str) -> String {
        extract_drive_id(provider_id).unwrap()
    }
}

/// Extract Drive file ID from `gdrive://<id>/<name>` format.
/// Also accepts a raw ID without the prefix.
fn extract_drive_id(provider_id: &str) -> Result<String, SourceError> {
    if let Some(rest) = provider_id.strip_prefix("gdrive://") {
        if let Some(slash) = rest.find('/') {
            Ok(rest[..slash].to_string())
        } else {
            Ok(rest.to_string())
        }
    } else {
        Ok(provider_id.to_string())
    }
}
