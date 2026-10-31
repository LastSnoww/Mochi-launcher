use std::path::PathBuf;

use tauri::{AppHandle, Manager};

/// The shared data folder (spec 6.6): `%APPDATA%/Mochi Client/` on Windows,
/// the platform data directory elsewhere.
#[derive(Clone)]
pub struct Paths {
    pub root: PathBuf,
}

impl Paths {
    pub fn resolve(app: &AppHandle) -> tauri::Result<Self> {
        Ok(Self {
            root: app.path().data_dir()?.join("Mochi Client"),
        })
    }

    pub fn accounts_file(&self) -> PathBuf {
        self.root.join("accounts.json")
    }

    pub fn skin_cache(&self) -> PathBuf {
        self.root.join("cache").join("skins")
    }
}
