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

## License

[GPL-3.0](LICENSE).
