use std::time::Duration;

use reqwest::{Client, Url, redirect};

/// Spec 6.8: `<GITHUB_REPO>/<launcher version>`. Used for every request, not only Modrinth.
pub const USER_AGENT: &str = concat!("LastSnoww/Mochi-launcher/", env!("CARGO_PKG_VERSION"));

/// Hosts the launcher may contact. Extended as later phases add Mojang downloads, loaders,
/// Modrinth and GitHub (spec 6.3).
const ALLOWED_HOSTS: &[&str] = &[
    "login.microsoftonline.com",
    "user.auth.xboxlive.com",
    "xsts.auth.xboxlive.com",
    "api.minecraftservices.com",
    "textures.minecraft.net",
];

pub fn is_allowed(url: &Url) -> bool {
    url.scheme() == "https"
        && url
            .host_str()
            .is_some_and(|host| ALLOWED_HOSTS.contains(&host))
}

/// Parses a URL that came from a remote response and checks it against the allowlist.
/// Plain `http://` is upgraded to `https://` first (skin URLs are served as http).
pub fn allowed_url(raw: &str) -> Option<Url> {
    let mut url = Url::parse(raw).ok()?;
    if url.scheme() == "http" {
        url.set_scheme("https").ok()?;
    }
    is_allowed(&url).then_some(url)
}

/// The shared HTTP client. Redirects are only followed to allowed hosts.
pub fn client() -> reqwest::Result<Client> {
    Client::builder()
        .user_agent(USER_AGENT)
        .connect_timeout(Duration::from_secs(15))
        .timeout(Duration::from_secs(60))
        .redirect(redirect::Policy::custom(|attempt| {
            if attempt.previous().len() >= 5 {
                attempt.error("too many redirects")
            } else if is_allowed(attempt.url()) {
                attempt.follow()
            } else {
                attempt.error("redirect to a host that is not on the allowlist")
            }
        }))
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allows_only_listed_https_hosts() {
        assert!(allowed_url("https://api.minecraftservices.com/minecraft/profile").is_some());
        assert!(allowed_url("https://evil.example.com/").is_none());
        assert!(allowed_url("https://api.minecraftservices.com.evil.example/").is_none());
        assert!(allowed_url("ftp://textures.minecraft.net/texture/abc").is_none());
        assert!(allowed_url("not a url").is_none());
    }

    #[test]
    fn upgrades_http_skin_urls() {
        let url = allowed_url("http://textures.minecraft.net/texture/abc123").unwrap();
        assert_eq!(
            url.as_str(),
            "https://textures.minecraft.net/texture/abc123"
        );
    }
}
