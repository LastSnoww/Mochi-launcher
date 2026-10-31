//! Microsoft OAuth 2.0 device code flow and token refresh.

use std::time::{Duration, Instant};

use reqwest::{Client, Response};
use serde::Deserialize;

use super::{AuthError, CLIENT_ID};
use crate::secret::Secret;

const AUTHORITY: &str = "https://login.microsoftonline.com/consumers/oauth2/v2.0";
const SCOPE: &str = "XboxLive.signin offline_access";
const DEVICE_CODE_GRANT: &str = "urn:ietf:params:oauth:grant-type:device_code";

#[derive(Debug, Deserialize)]
pub struct DeviceCode {
    pub device_code: Secret,
    pub user_code: String,
    pub verification_uri: String,
    pub expires_in: u64,
    #[serde(default = "default_interval")]
    pub interval: u64,
}

fn default_interval() -> u64 {
    5
}

#[derive(Debug, Deserialize)]
pub struct MsaTokens {
    pub access_token: Secret,
    pub refresh_token: Option<Secret>,
}

#[derive(Debug, Deserialize)]
struct OAuthError {
    error: String,
    #[serde(default)]
    error_description: Option<String>,
}

impl OAuthError {
    fn into_auth_error(self) -> AuthError {
        match self.error.as_str() {
            "authorization_declined" | "access_denied" => AuthError::Declined,
            "expired_token" => AuthError::CodeExpired,
            "invalid_grant" => AuthError::SessionExpired,
            _ => {
                // The description's first line carries the AADSTS code, which is what
                // explains client ID problems. It never contains tokens.
                let detail = self
                    .error_description
                    .as_deref()
                    .and_then(|d| d.lines().next())
                    .unwrap_or_default();
                AuthError::Microsoft(format!("{} {}", self.error, detail).trim().to_string())
            }
        }
    }
}

async fn error_from(resp: Response) -> AuthError {
    let status = resp.status().as_u16();
    match resp.json::<OAuthError>().await {
        Ok(err) => err.into_auth_error(),
        Err(_) => AuthError::Unexpected {
            service: "Microsoft",
            status,
        },
    }
}

pub async fn request_device_code(http: &Client) -> Result<DeviceCode, AuthError> {
    let resp = http
        .post(format!("{AUTHORITY}/devicecode"))
        .form(&[("client_id", CLIENT_ID), ("scope", SCOPE)])
        .send()
        .await?;
    if !resp.status().is_success() {
        return Err(error_from(resp).await);
    }
    Ok(resp.json().await?)
}

/// Polls until the player approves the code, declines it, or it expires.
pub async fn poll_for_tokens(http: &Client, code: &DeviceCode) -> Result<MsaTokens, AuthError> {
    let deadline = Instant::now() + Duration::from_secs(code.expires_in);
    let mut interval = Duration::from_secs(code.interval.max(1));
    loop {
        tokio::time::sleep(interval).await;
        if Instant::now() >= deadline {
            return Err(AuthError::CodeExpired);
        }
        let resp = http
            .post(format!("{AUTHORITY}/token"))
            .form(&[
                ("grant_type", DEVICE_CODE_GRANT),
                ("client_id", CLIENT_ID),
                ("device_code", code.device_code.expose()),
            ])
            .send()
            .await?;
        if resp.status().is_success() {
            return Ok(resp.json().await?);
        }
        let status = resp.status().as_u16();
        let err: OAuthError = resp.json().await.map_err(|_| AuthError::Unexpected {
            service: "Microsoft",
            status,
        })?;
        match err.error.as_str() {
            "authorization_pending" => {}
            "slow_down" => interval += Duration::from_secs(5),
            _ => return Err(err.into_auth_error()),
        }
    }
}

pub async fn refresh(http: &Client, refresh_token: &Secret) -> Result<MsaTokens, AuthError> {
    let resp = http
        .post(format!("{AUTHORITY}/token"))
        .form(&[
            ("grant_type", "refresh_token"),
            ("client_id", CLIENT_ID),
            ("refresh_token", refresh_token.expose()),
            ("scope", SCOPE),
        ])
        .send()
        .await?;
    if !resp.status().is_success() {
        return Err(error_from(resp).await);
    }
    Ok(resp.json().await?)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(json: &str) -> AuthError {
        serde_json::from_str::<OAuthError>(json)
            .unwrap()
            .into_auth_error()
    }

    #[test]
    fn maps_oauth_errors() {
        assert!(matches!(
            parse(r#"{"error":"invalid_grant"}"#),
            AuthError::SessionExpired
        ));
        assert!(matches!(
            parse(r#"{"error":"expired_token"}"#),
            AuthError::CodeExpired
        ));
        assert!(matches!(
            parse(r#"{"error":"authorization_declined"}"#),
            AuthError::Declined
        ));
        let other = parse(
            r#"{"error":"unauthorized_client","error_description":"AADSTS700016: App not found.\r\nTrace ID: x"}"#,
        );
        assert_eq!(
            other.to_string(),
            "Microsoft sign-in failed: unauthorized_client AADSTS700016: App not found."
        );
    }

    #[test]
    fn parses_device_code_response() {
        let code: DeviceCode = serde_json::from_str(
            r#"{"user_code":"ABCD1234","device_code":"long-secret","verification_uri":"https://www.microsoft.com/link","expires_in":900,"interval":5,"message":"To sign in, use a web browser..."}"#,
        )
        .unwrap();
        assert_eq!(code.user_code, "ABCD1234");
        assert_eq!(code.interval, 5);
        assert!(!format!("{code:?}").contains("long-secret"));
    }
}
