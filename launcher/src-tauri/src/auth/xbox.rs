//! Xbox Live user authentication and XSTS authorization for the Minecraft relying party.

use reqwest::{Client, StatusCode, header::ACCEPT};
use serde::Deserialize;
use serde_json::json;

use super::AuthError;
use crate::secret::Secret;

const USER_AUTH_URL: &str = "https://user.auth.xboxlive.com/user/authenticate";
const XSTS_URL: &str = "https://xsts.auth.xboxlive.com/xsts/authorize";

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct XboxResponse {
    token: Secret,
    display_claims: DisplayClaims,
}

#[derive(Deserialize)]
struct DisplayClaims {
    xui: Vec<UserClaims>,
}

#[derive(Deserialize)]
struct UserClaims {
    uhs: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct XboxError {
    x_err: u64,
}

pub struct XstsToken {
    pub token: Secret,
    pub user_hash: String,
}

pub async fn authenticate(http: &Client, ms_access_token: &Secret) -> Result<Secret, AuthError> {
    let body = json!({
        "Properties": {
            "AuthMethod": "RPS",
            "SiteName": "user.auth.xboxlive.com",
            "RpsTicket": format!("d={}", ms_access_token.expose()),
        },
        "RelyingParty": "http://auth.xboxlive.com",
        "TokenType": "JWT",
    });
    let resp = http
        .post(USER_AUTH_URL)
        .header(ACCEPT, "application/json")
        .json(&body)
        .send()
        .await?;
    if !resp.status().is_success() {
        return Err(AuthError::Unexpected {
            service: "Xbox Live",
            status: resp.status().as_u16(),
        });
    }
    Ok(resp.json::<XboxResponse>().await?.token)
}

pub async fn authorize(http: &Client, xbl_token: &Secret) -> Result<XstsToken, AuthError> {
    let body = json!({
        "Properties": {
            "SandboxId": "RETAIL",
            "UserTokens": [xbl_token.expose()],
        },
        "RelyingParty": "rp://api.minecraftservices.com/",
        "TokenType": "JWT",
    });
    let resp = http
        .post(XSTS_URL)
        .header(ACCEPT, "application/json")
        .json(&body)
        .send()
        .await?;
    let status = resp.status();
    if status == StatusCode::UNAUTHORIZED {
        if let Ok(err) = resp.json::<XboxError>().await {
            return Err(AuthError::Xbox(xerr_message(err.x_err)));
        }
    } else if status.is_success() {
        let parsed: XboxResponse = resp.json().await?;
        let user_hash = parsed
            .display_claims
            .xui
            .into_iter()
            .next()
            .map(|claims| claims.uhs)
            .ok_or(AuthError::Unexpected {
                service: "Xbox Live",
                status: status.as_u16(),
            })?;
        return Ok(XstsToken {
            token: parsed.token,
            user_hash,
        });
    }
    Err(AuthError::Unexpected {
        service: "Xbox Live",
        status: status.as_u16(),
    })
}

/// Friendly messages for the XSTS `XErr` codes listed in spec 6.1.
pub fn xerr_message(code: u64) -> String {
    match code {
        2148916233 => "This Microsoft account has no Xbox profile yet. Sign in once at xbox.com to create one, then try again.".into(),
        2148916235 => "Xbox Live isn't available in your country or region, so this account can't sign in to Minecraft.".into(),
        2148916238 => "This is a child account. An adult needs to add it to a Microsoft family group before it can play.".into(),
        other => format!("Xbox Live refused the sign-in (error {other})."),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_xsts_error_body() {
        let err: XboxError = serde_json::from_str(
            r#"{"Identity":"0","XErr":2148916233,"Message":"","Redirect":"https://start.ui.xboxlive.com/CreateAccount"}"#,
        )
        .unwrap();
        assert_eq!(err.x_err, 2148916233);
        assert!(xerr_message(err.x_err).contains("no Xbox profile"));
    }

    #[test]
    fn known_and_unknown_xerr_codes() {
        assert!(xerr_message(2148916235).contains("country"));
        assert!(xerr_message(2148916238).contains("family"));
        assert_eq!(xerr_message(1), "Xbox Live refused the sign-in (error 1).");
    }

    #[test]
    fn parses_xsts_success_body() {
        let parsed: XboxResponse = serde_json::from_str(
            r#"{"IssueInstant":"2026-10-31T00:00:00Z","NotAfter":"2026-11-01T00:00:00Z","Token":"xsts-secret","DisplayClaims":{"xui":[{"uhs":"1234567890"}]}}"#,
        )
        .unwrap();
        assert_eq!(parsed.display_claims.xui[0].uhs, "1234567890");
        assert_eq!(parsed.token.expose(), "xsts-secret");
    }
}
