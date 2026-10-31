//! Account list in `accounts.json` (no tokens) and refresh tokens in the OS keychain.

use std::fs;
use std::io;
use std::path::Path;

use serde::{Deserialize, Serialize};

use super::AuthError;
use crate::secret::Secret;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    /// Minecraft profile UUID without dashes.
    pub uuid: String,
    pub name: String,
    #[serde(default)]
    pub skin_url: Option<String>,
}

#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountsFile {
    #[serde(default)]
    pub active: Option<String>,
    #[serde(default)]
    pub accounts: Vec<Account>,
}

impl AccountsFile {
    /// A missing file means no accounts. A corrupt file is kept as `.bak` and replaced by an
    /// empty list; the refresh tokens stay in the keychain until the account signs in again.
    pub fn load(path: &Path) -> io::Result<Self> {
        match fs::read(path) {
            Ok(bytes) => match serde_json::from_slice(&bytes) {
                Ok(file) => Ok(file),
                Err(_) => {
                    fs::rename(path, path.with_extension("json.bak"))?;
                    Ok(Self::default())
                }
            },
            Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(Self::default()),
            Err(err) => Err(err),
        }
    }

    /// Writes to a temp file and renames it into place.
    pub fn save(&self, path: &Path) -> io::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let tmp = path.with_extension("json.tmp");
        fs::write(
            &tmp,
            serde_json::to_vec_pretty(self).map_err(io::Error::other)?,
        )?;
        fs::rename(&tmp, path)
    }

    pub fn get(&self, uuid: &str) -> Option<&Account> {
        self.accounts.iter().find(|a| a.uuid == uuid)
    }

    /// Adds the account or updates its name and skin, and makes it active.
    pub fn upsert_active(&mut self, account: Account) {
        self.active = Some(account.uuid.clone());
        match self.accounts.iter_mut().find(|a| a.uuid == account.uuid) {
            Some(existing) => *existing = account,
            None => self.accounts.push(account),
        }
    }

    pub fn set_active(&mut self, uuid: &str) -> bool {
        let known = self.get(uuid).is_some();
        if known {
            self.active = Some(uuid.to_string());
        }
        known
    }

    /// Removes the account. If it was active, the first remaining account becomes active.
    pub fn remove(&mut self, uuid: &str) -> bool {
        let before = self.accounts.len();
        self.accounts.retain(|a| a.uuid != uuid);
        if self.active.as_deref() == Some(uuid) {
            self.active = self.accounts.first().map(|a| a.uuid.clone());
        }
        self.accounts.len() != before
    }
}

/// Profile UUIDs are 32 hex characters. Checked before one is used as a keychain key.
pub fn is_valid_uuid(uuid: &str) -> bool {
    uuid.len() == 32 && uuid.bytes().all(|b| b.is_ascii_hexdigit())
}

const KEYCHAIN_SERVICE: &str = "Mochi Client";
/// Windows Credential Manager limits one secret to 2560 bytes, so longer tokens are split.
const CHUNK_BYTES: usize = 2048;
const MAX_CHUNKS: usize = 8;

fn keychain_error(err: keyring::Error) -> AuthError {
    AuthError::Keychain(err.to_string())
}

fn chunk_entry(uuid: &str, index: usize) -> Result<keyring::Entry, AuthError> {
    keyring::Entry::new(KEYCHAIN_SERVICE, &format!("{uuid}/refresh/{index}"))
        .map_err(keychain_error)
}

fn delete_chunks_from(uuid: &str, first: usize) -> Result<(), AuthError> {
    for index in first..MAX_CHUNKS {
        match chunk_entry(uuid, index)?.delete_credential() {
            Ok(()) => {}
            Err(keyring::Error::NoEntry) => break,
            Err(err) => return Err(keychain_error(err)),
        }
    }
    Ok(())
}

pub fn save_refresh_token(uuid: &str, token: &Secret) -> Result<(), AuthError> {
    let chunks: Vec<&[u8]> = token.expose().as_bytes().chunks(CHUNK_BYTES).collect();
    if chunks.len() > MAX_CHUNKS {
        return Err(AuthError::Keychain("refresh token is too large".into()));
    }
    for (index, chunk) in chunks.iter().enumerate() {
        chunk_entry(uuid, index)?
            .set_secret(chunk)
            .map_err(keychain_error)?;
    }
    delete_chunks_from(uuid, chunks.len())
}

pub fn load_refresh_token(uuid: &str) -> Result<Option<Secret>, AuthError> {
    let mut bytes = Vec::new();
    for index in 0..MAX_CHUNKS {
        match chunk_entry(uuid, index)?.get_secret() {
            Ok(chunk) => bytes.extend(chunk),
            Err(keyring::Error::NoEntry) => break,
            Err(err) => return Err(keychain_error(err)),
        }
    }
    if bytes.is_empty() {
        return Ok(None);
    }
    String::from_utf8(bytes)
        .map(|token| Some(Secret::new(token)))
        .map_err(|_| AuthError::Keychain("stored token is not valid text".into()))
}

pub fn delete_refresh_token(uuid: &str) -> Result<(), AuthError> {
    delete_chunks_from(uuid, 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn account(uuid: &str, name: &str) -> Account {
        Account {
            uuid: uuid.into(),
            name: name.into(),
            skin_url: None,
        }
    }

    #[test]
    fn upsert_switch_and_remove() {
        let mut file = AccountsFile::default();
        file.upsert_active(account("a", "Alice"));
        file.upsert_active(account("b", "Bob"));
        assert_eq!(file.active.as_deref(), Some("b"));

        file.upsert_active(account("a", "Alice2"));
        assert_eq!(file.accounts.len(), 2);
        assert_eq!(file.get("a").unwrap().name, "Alice2");

        assert!(file.set_active("b"));
        assert!(!file.set_active("nobody"));
        assert_eq!(file.active.as_deref(), Some("b"));

        assert!(file.remove("b"));
        assert_eq!(file.active.as_deref(), Some("a"));
        assert!(file.remove("a"));
        assert_eq!(file.active, None);
        assert!(!file.remove("a"));
    }

    #[test]
    fn save_and_load_round_trip_and_recover_from_corruption() {
        let dir = std::env::temp_dir().join(format!("mochi-accounts-test-{}", std::process::id()));
        let path = dir.join("accounts.json");
        assert_eq!(AccountsFile::load(&path).unwrap(), AccountsFile::default());

        let mut file = AccountsFile::default();
        file.upsert_active(account("069a79f444e94726a5befca90e38aaf5", "Notch"));
        file.save(&path).unwrap();
        assert_eq!(AccountsFile::load(&path).unwrap(), file);
        assert!(!fs::read_to_string(&path).unwrap().contains("token"));

        fs::write(&path, "{ not json").unwrap();
        assert_eq!(AccountsFile::load(&path).unwrap(), AccountsFile::default());
        assert!(path.with_extension("json.bak").exists());
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn validates_uuids() {
        assert!(is_valid_uuid("069a79f444e94726a5befca90e38aaf5"));
        assert!(!is_valid_uuid("069a79f4-44e9-4726-a5be-fca90e38aaf5"));
        assert!(!is_valid_uuid("../../etc/passwd000000000000000000"));
        assert!(!is_valid_uuid(""));
    }

    /// Uses the real OS keychain, so it only runs on request: `cargo test -- --ignored`.
    #[test]
    #[ignore]
    fn keychain_round_trip_with_chunking() {
        let uuid = "00000000000000000000000000000000";
        let long = Secret::new("x".repeat(CHUNK_BYTES * 2 + 17));
        save_refresh_token(uuid, &long).unwrap();
        assert_eq!(load_refresh_token(uuid).unwrap(), Some(long));

        // A shorter token must not leave stale chunks behind.
        let short = Secret::new("short".into());
        save_refresh_token(uuid, &short).unwrap();
        assert_eq!(load_refresh_token(uuid).unwrap(), Some(short));

        delete_refresh_token(uuid).unwrap();
        assert_eq!(load_refresh_token(uuid).unwrap(), None);
    }
}
