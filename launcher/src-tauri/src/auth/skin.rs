//! Downloads and caches player skins so the UI can draw heads without contacting any
//! third-party head service. Skin URLs are content-addressed, so cached files never go stale.

use std::fs;
use std::path::Path;

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use reqwest::Client;

use super::AuthError;
use crate::net;

const MAX_SKIN_BYTES: usize = 256 * 1024;
const PNG_SIGNATURE: &[u8] = b"\x89PNG\r\n\x1a\n";

/// The texture hash at the end of a skin URL, if it looks like one.
fn texture_hash(url: &reqwest::Url) -> Option<&str> {
    url.path_segments()?
        .next_back()
        .filter(|hash| !hash.is_empty() && hash.len() <= 128)
        .filter(|hash| hash.bytes().all(|b| b.is_ascii_hexdigit()))
}

/// Returns the skin as a `data:image/png;base64,...` URL.
pub async fn skin_data_url(
    http: &Client,
    cache_dir: &Path,
    skin_url: &str,
) -> Result<String, AuthError> {
    let url = net::allowed_url(skin_url).ok_or(AuthError::InvalidSkin)?;
    let hash = texture_hash(&url).ok_or(AuthError::InvalidSkin)?;
    let path = cache_dir.join(format!("{hash}.png"));

    let bytes = match fs::read(&path) {
        Ok(bytes) => bytes,
        Err(_) => {
            let resp = http.get(url.clone()).send().await?;
            if !resp.status().is_success() {
                return Err(AuthError::Unexpected {
                    service: "Minecraft skins",
                    status: resp.status().as_u16(),
                });
            }
            let bytes = resp.bytes().await?.to_vec();
            if bytes.len() > MAX_SKIN_BYTES || !bytes.starts_with(PNG_SIGNATURE) {
                return Err(AuthError::InvalidSkin);
            }
            fs::create_dir_all(cache_dir)?;
            let tmp = path.with_extension("png.tmp");
            fs::write(&tmp, &bytes)?;
            fs::rename(&tmp, &path)?;
            bytes
        }
    };
    Ok(format!("data:image/png;base64,{}", STANDARD.encode(bytes)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_texture_hash() {
        let url = net::allowed_url("http://textures.minecraft.net/texture/3b60a1f6d562f52aaebbf1434f1de147933a3affe0e764fa49ea057536623cd3").unwrap();
        assert_eq!(
            texture_hash(&url),
            Some("3b60a1f6d562f52aaebbf1434f1de147933a3affe0e764fa49ea057536623cd3")
        );
        let bad = net::allowed_url("https://textures.minecraft.net/texture/..%2F..%2Fx").unwrap();
        assert_eq!(texture_hash(&bad), None);
    }
}
