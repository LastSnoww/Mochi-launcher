mod auth;
mod net;
mod paths;
mod secret;

use tauri::Manager;

use auth::commands::{self as auth_commands, AuthState};

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let paths = paths::Paths::resolve(app.handle())?;
            let http = net::client()?;
            app.manage(AuthState::new(http, paths)?);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            auth_commands::accounts_list,
            auth_commands::account_switch,
            auth_commands::account_remove,
            auth_commands::account_refresh,
            auth_commands::account_head,
            auth_commands::auth_login_start,
            auth_commands::auth_login_cancel,
            auth_commands::auth_open_verification,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Mochi Client");
}
