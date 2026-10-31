// Every app command is listed here so the UI can only call the ones a capability allows.
const COMMANDS: &[&str] = &[
    "accounts_list",
    "account_switch",
    "account_remove",
    "account_refresh",
    "account_head",
    "auth_login_start",
    "auth_login_cancel",
    "auth_open_verification",
];

fn main() {
    tauri_build::try_build(
        tauri_build::Attributes::new()
            .app_manifest(tauri_build::AppManifest::new().commands(COMMANDS)),
    )
    .expect("failed to run tauri-build");
}
