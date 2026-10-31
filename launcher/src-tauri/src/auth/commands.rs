//! Tauri commands and state for accounts. Login progress is reported with `auth:login` events.

use reqwest::{Client, Url};
use serde::Serialize;
use tauri::async_runtime::{self, JoinHandle};
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_opener::OpenerExt;
use tokio::sync::Mutex;

use super::store::{self, Account, AccountsFile};
use super::{AuthError, minecraft_profile, msa, skin};
use crate::paths::Paths;

pub struct AuthState {
    http: Client,
    paths: Paths,
    inner: Mutex<Inner>,
}

struct Inner {
    file: AccountsFile,
    login: Option<PendingLogin>,
}

struct PendingLogin {
    verification_uri: String,
    task: JoinHandle<()>,
}

impl AuthState {
    pub fn new(http: Client, paths: Paths) -> std::io::Result<Self> {
        let file = AccountsFile::load(&paths.accounts_file())?;
        Ok(Self {
            http,
            paths,
            inner: Mutex::new(Inner { file, login: None }),
        })
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandError {
    kind: &'static str,
    message: String,
}

impl From<AuthError> for CommandError {
    fn from(err: AuthError) -> Self {
        let kind = match err {
            AuthError::SessionExpired => "session-expired",
            AuthError::Network(_) => "network",
            _ => "error",
        };
        Self {
            kind,
            message: err.to_string(),
        }
    }
}

type CommandResult<T> = Result<T, CommandError>;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountsView {
    active: Option<String>,
    accounts: Vec<Account>,
}

impl From<&AccountsFile> for AccountsView {
    fn from(file: &AccountsFile) -> Self {
        Self {
            active: file.active.clone(),
            accounts: file.accounts.clone(),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceCodeView {
    user_code: String,
    verification_uri: String,
    expires_in: u64,
}

#[derive(Clone, Serialize)]
#[serde(tag = "status", rename_all = "kebab-case")]
enum LoginEvent {
    SigningIn,
    Done { uuid: String },
    Error { message: String },
}

const LOGIN_EVENT: &str = "auth:login";

fn known_uuid(file: &AccountsFile, uuid: &str) -> Result<(), AuthError> {
    if store::is_valid_uuid(uuid) && file.get(uuid).is_some() {
        Ok(())
    } else {
        Err(AuthError::UnknownAccount)
    }
}

#[tauri::command]
pub async fn accounts_list(state: State<'_, AuthState>) -> CommandResult<AccountsView> {
    Ok(AccountsView::from(&state.inner.lock().await.file))
}

#[tauri::command]
pub async fn account_switch(
    state: State<'_, AuthState>,
    uuid: String,
) -> CommandResult<AccountsView> {
    let mut inner = state.inner.lock().await;
    known_uuid(&inner.file, &uuid)?;
    inner.file.set_active(&uuid);
    inner
        .file
        .save(&state.paths.accounts_file())
        .map_err(AuthError::from)?;
    Ok(AccountsView::from(&inner.file))
}

#[tauri::command]
pub async fn account_remove(
    state: State<'_, AuthState>,
    uuid: String,
) -> CommandResult<AccountsView> {
    let mut inner = state.inner.lock().await;
    known_uuid(&inner.file, &uuid)?;
    store::delete_refresh_token(&uuid)?;
    inner.file.remove(&uuid);
    inner
        .file
        .save(&state.paths.accounts_file())
        .map_err(AuthError::from)?;
    Ok(AccountsView::from(&inner.file))
}

/// Refreshes the Microsoft token and re-reads the profile, so name and skin changes show up.
/// Fails with kind `session-expired` when the player has to sign in again.
#[tauri::command]
pub async fn account_refresh(
    state: State<'_, AuthState>,
    uuid: String,
) -> CommandResult<AccountsView> {
    known_uuid(&state.inner.lock().await.file, &uuid)?;
    let refresh_token = store::load_refresh_token(&uuid)?.ok_or(AuthError::SessionExpired)?;
    let tokens = msa::refresh(&state.http, &refresh_token).await?;
    if let Some(new_refresh) = &tokens.refresh_token {
        store::save_refresh_token(&uuid, new_refresh)?;
    }
    let profile = minecraft_profile(&state.http, &tokens.access_token).await?;

    let mut inner = state.inner.lock().await;
    if let Some(account) = inner.file.accounts.iter_mut().find(|a| a.uuid == uuid) {
        account.name = profile.name.clone();
        account.skin_url = profile.active_skin_url().map(str::to_string);
    }
    inner
        .file
        .save(&state.paths.accounts_file())
        .map_err(AuthError::from)?;
    Ok(AccountsView::from(&inner.file))
}

/// The account's skin as a data URL, or `None` for accounts without a custom skin.
#[tauri::command]
pub async fn account_head(
    state: State<'_, AuthState>,
    uuid: String,
) -> CommandResult<Option<String>> {
    let skin_url = {
        let inner = state.inner.lock().await;
        known_uuid(&inner.file, &uuid)?;
        inner.file.get(&uuid).and_then(|a| a.skin_url.clone())
    };
    match skin_url {
        Some(url) => Ok(Some(
            skin::skin_data_url(&state.http, &state.paths.skin_cache(), &url).await?,
        )),
        None => Ok(None),
    }
}

#[tauri::command]
pub async fn auth_login_start(
    app: AppHandle,
    state: State<'_, AuthState>,
) -> CommandResult<DeviceCodeView> {
    let mut inner = state.inner.lock().await;
    if let Some(previous) = inner.login.take() {
        previous.task.abort();
    }
    let code = msa::request_device_code(&state.http).await?;
    let view = DeviceCodeView {
        user_code: code.user_code.clone(),
        verification_uri: code.verification_uri.clone(),
        expires_in: code.expires_in,
    };

    // The lock is held until the task is stored, so the task can't clear `login` first.
    let task_app = app.clone();
    let task = async_runtime::spawn(async move {
        let event = match finish_login(&task_app, &code).await {
            Ok(uuid) => LoginEvent::Done { uuid },
            Err(err) => LoginEvent::Error {
                message: err.to_string(),
            },
        };
        task_app.state::<AuthState>().inner.lock().await.login = None;
        let _ = task_app.emit(LOGIN_EVENT, event);
    });
    inner.login = Some(PendingLogin {
        verification_uri: view.verification_uri.clone(),
        task,
    });
    Ok(view)
}

async fn finish_login(app: &AppHandle, code: &msa::DeviceCode) -> Result<String, AuthError> {
    let state = app.state::<AuthState>();
    let tokens = msa::poll_for_tokens(&state.http, code).await?;
    let _ = app.emit(LOGIN_EVENT, LoginEvent::SigningIn);
    let profile = minecraft_profile(&state.http, &tokens.access_token).await?;
    if !store::is_valid_uuid(&profile.id) {
        return Err(AuthError::Unexpected {
            service: "Minecraft",
            status: 200,
        });
    }
    let refresh_token = tokens
        .refresh_token
        .ok_or(AuthError::Microsoft("no refresh token was returned".into()))?;
    store::save_refresh_token(&profile.id, &refresh_token)?;

    let mut inner = state.inner.lock().await;
    inner.file.upsert_active(Account {
        uuid: profile.id.clone(),
        name: profile.name.clone(),
        skin_url: profile.active_skin_url().map(str::to_string),
    });
    inner.file.save(&state.paths.accounts_file())?;
    Ok(profile.id)
}

#[tauri::command]
pub async fn auth_login_cancel(state: State<'_, AuthState>) -> CommandResult<()> {
    if let Some(login) = state.inner.lock().await.login.take() {
        login.task.abort();
    }
    Ok(())
}

/// Opens the pending login's verification page in the default browser. Only Microsoft
/// sign-in pages are opened, whatever the device code response said.
#[tauri::command]
pub async fn auth_open_verification(
    app: AppHandle,
    state: State<'_, AuthState>,
) -> CommandResult<()> {
    let uri = state
        .inner
        .lock()
        .await
        .login
        .as_ref()
        .map(|login| login.verification_uri.clone())
        .ok_or(AuthError::CodeExpired)?;
    if !is_microsoft_login_page(&uri) {
        return Err(AuthError::Microsoft("unexpected verification address".into()).into());
    }
    app.opener()
        .open_url(uri, None::<&str>)
        .map_err(|err| AuthError::Microsoft(err.to_string()))?;
    Ok(())
}

fn is_microsoft_login_page(uri: &str) -> bool {
    const HOSTS: &[&str] = &[
        "www.microsoft.com",
        "microsoft.com",
        "login.microsoftonline.com",
        "login.live.com",
    ];
    Url::parse(uri).is_ok_and(|url| {
        url.scheme() == "https" && url.host_str().is_some_and(|host| HOSTS.contains(&host))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_opens_microsoft_pages() {
        assert!(is_microsoft_login_page("https://www.microsoft.com/link"));
        assert!(is_microsoft_login_page(
            "https://login.microsoftonline.com/common/oauth2/deviceauth"
        ));
        assert!(!is_microsoft_login_page("http://www.microsoft.com/link"));
        assert!(!is_microsoft_login_page(
            "https://microsoft.com.evil.example/"
        ));
        assert!(!is_microsoft_login_page(
            "file:///C:/Windows/System32/calc.exe"
        ));
    }

    #[test]
    fn session_expired_has_its_own_kind() {
        let err = CommandError::from(AuthError::SessionExpired);
        assert_eq!(err.kind, "session-expired");
    }
}
