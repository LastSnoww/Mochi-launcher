# Mochi Client

A lightweight, open-source Minecraft launcher with a soft pastel UI and ready-made PvP presets.

No telemetry, no accounts of ours, no server of ours.

> Early development. Phase 1 (skeleton and design system) only: the launcher opens with mock data and cannot launch the game yet.

## Building

Requirements: Node 22+, pnpm (`corepack enable pnpm`), Rust stable, and on Windows the WebView2 runtime (preinstalled on Windows 11).

```sh
cd launcher
pnpm install
pnpm tauri dev      # run the app
pnpm test           # contrast test and other unit tests
pnpm check          # svelte-check
pnpm lint           # eslint
```

## Microsoft sign-in for forks

Sign-in uses a public Azure app registration (device code flow, no client secret). If you fork this launcher, register your own app in the Azure portal, get it approved for the Minecraft API, and put its client ID in `launcher/src-tauri/src/auth/mod.rs` (`CLIENT_ID`). Don't ship a fork with someone else's client ID.

Refresh tokens are stored in the OS keychain (Windows Credential Manager, macOS Keychain, Secret Service on Linux), never in files.

## License

[GPL-3.0](LICENSE).
