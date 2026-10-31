//! Microsoft → Xbox Live → XSTS → Minecraft sign-in (spec 6.1).

pub mod commands;
mod minecraft;
mod msa;
mod skin;
mod store;
mod xbox;

use reqwest::Client;

use crate::secret::Secret;
use minecraft::Profile;

/// Azure app registration (public client, no secret). Forks must register their own.
/// TODO(before release): this is a stand-in; replace it with the project's own client ID
/// once it is approved for the Minecraft API.
pub const CLIENT_ID: &str = "04b07795-8ddb-461a-bbee-02f9e1bf7b46";

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Couldn't reach the sign-in service. Check your internet connection and try again.")]
    Network(#[from] reqwest::Error),
    #[error("The sign-in code expired. Start again to get a new one.")]
    CodeExpired,
    #[error("Sign-in was declined.")]
    Declined,
    #[error("Your Microsoft sign-in has expired. Sign in again.")]
    SessionExpired,
    #[error("Microsoft sign-in failed: {0}")]
    Microsoft(String),
    #[error("{0}")]
    Xbox(String),
    #[error(
        "Minecraft rejected this launcher's app registration. The Azure client ID isn't approved for the Minecraft API yet."
    )]
    AppNotApproved,
    #[error("Minecraft's servers are busy. Wait a minute and try again.")]
    RateLimited,
    #[error("This Microsoft account doesn't own Minecraft: Java Edition.")]
    NotOwned,
    #[error(
        "This account has no Minecraft profile yet. Set one up at minecraft.net, then try again."
    )]
    NoProfile,
    #[error("Unexpected response from {service} (HTTP {status}).")]
    Unexpected { service: &'static str, status: u16 },
    #[error("The skin file looks invalid.")]
    InvalidSkin,
    #[error("Couldn't use the system keychain: {0}")]
    Keychain(String),
    #[error("Couldn't save account data: {0}")]
    Io(#[from] std::io::Error),
    #[error("That account isn't signed in on this launcher.")]
    UnknownAccount,
}

/// Runs the Xbox and Minecraft steps for a Microsoft access token and returns the player's
/// profile. Ownership is checked through the entitlements endpoint when there is no profile,
/// so the error tells the player whether to buy the game or create a profile.
async fn minecraft_profile(http: &Client, ms_access_token: &Secret) -> Result<Profile, AuthError> {
    let xbl = xbox::authenticate(http, ms_access_token).await?;
    let xsts = xbox::authorize(http, &xbl).await?;
    let mc_token = minecraft::login_with_xbox(http, &xsts).await?;
    match minecraft::profile(http, &mc_token).await? {
        Some(profile) => Ok(profile),
        None if minecraft::owns_game(http, &mc_token).await? => Err(AuthError::NoProfile),
        None => Err(AuthError::NotOwned),
    }
}
