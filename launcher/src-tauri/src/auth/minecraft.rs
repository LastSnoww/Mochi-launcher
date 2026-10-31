//! Minecraft services: Xbox login, ownership and profile.

use reqwest::{Client, StatusCode};
use serde::Deserialize;
use serde_json::json;

use super::AuthError;
use super::xbox::XstsToken;
use crate::secret::Secret;

const API: &str = "https://api.minecraftservices.com";

#[derive(Deserialize)]
struct LoginResponse {
    access_token: Secret,
}

#[derive(Deserialize)]
struct Entitlements {
    #[serde(default)]
    items: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Profile {
    /// UUID without dashes.
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub skins: Vec<Skin>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Skin {
    pub state: String,
    pub url: String,
}

impl Profile {
    pub fn active_skin_url(&self) -> Option<&str> {
        self.skins
            .iter()
            .find(|skin| skin.state == "ACTIVE")
            .map(|skin| skin.url.as_str())
    }
}

fn unexpected(status: StatusCode) -> AuthError {
    match status {
        StatusCode::TOO_MANY_REQUESTS => AuthError::RateLimited,
        _ => AuthError::Unexpected {
            service: "Minecraft",
            status: status.as_u16(),
        },
    }
}

pub async fn login_with_xbox(http: &Client, xsts: &XstsToken) -> Result<Secret, AuthError> {
    let identity_token = format!("XBL3.0 x={};{}", xsts.user_hash, xsts.token.expose());
    let resp = http
        .post(format!("{API}/authentication/login_with_xbox"))
        .json(&json!({ "identityToken": identity_token }))
        .send()
        .await?;
    match resp.status() {
        status if status.is_success() => Ok(resp.json::<LoginResponse>().await?.access_token),
        // Returned when the Azure app isn't approved for the Minecraft API.
        StatusCode::FORBIDDEN => Err(AuthError::AppNotApproved),
        status => Err(unexpected(status)),
    }
}

pub async fn owns_game(http: &Client, token: &Secret) -> Result<bool, AuthError> {
    let resp = http
        .get(format!("{API}/entitlements/mcstore"))
        .bearer_auth(token.expose())
        .send()
        .await?;
    if !resp.status().is_success() {
        return Err(unexpected(resp.status()));
    }
    Ok(!resp.json::<Entitlements>().await?.items.is_empty())
}

/// `None` when the account has no Minecraft profile yet.
pub async fn profile(http: &Client, token: &Secret) -> Result<Option<Profile>, AuthError> {
    let resp = http
        .get(format!("{API}/minecraft/profile"))
        .bearer_auth(token.expose())
        .send()
        .await?;
    match resp.status() {
        StatusCode::NOT_FOUND => Ok(None),
        status if status.is_success() => Ok(Some(resp.json().await?)),
        status => Err(unexpected(status)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_profile_and_picks_active_skin() {
        let profile: Profile = serde_json::from_str(
            r#"{"id":"069a79f444e94726a5befca90e38aaf5","name":"Notch","skins":[
                {"id":"a","state":"INACTIVE","url":"http://textures.minecraft.net/texture/old","variant":"CLASSIC"},
                {"id":"b","state":"ACTIVE","url":"http://textures.minecraft.net/texture/new","variant":"SLIM"}
            ],"capes":[]}"#,
        )
        .unwrap();
        assert_eq!(profile.name, "Notch");
        assert_eq!(
            profile.active_skin_url(),
            Some("http://textures.minecraft.net/texture/new")
        );
    }

    #[test]
    fn profile_without_skins() {
        let profile: Profile =
            serde_json::from_str(r#"{"id":"069a79f444e94726a5befca90e38aaf5","name":"x"}"#)
                .unwrap();
        assert_eq!(profile.active_skin_url(), None);
    }

    #[test]
    fn empty_entitlements_mean_no_ownership() {
        let e: Entitlements =
            serde_json::from_str(r#"{"items":[],"signature":"s","keyId":"1"}"#).unwrap();
        assert!(e.items.is_empty());
    }
}
