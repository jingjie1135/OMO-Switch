# PROJECT KNOWLEDGE BASE

**Generated:** 2026-05-21 17:58 Asia/Shanghai
**Commit:** 762eb7c
**Branch:** main

## OVERVIEW

OMO Switch is a Tauri 2 desktop app for managing oh-my-openagent/OpenCode model configuration. Frontend is React 18 + TypeScript + Zustand + Tailwind; backend is Rust services exposed through Tauri IPC.

## STRUCTURE

```text
OMO-Switch/
├── src/                         # React app, stores, IPC wrappers, locale JSON
├── src-tauri/                   # Tauri app shell, Rust commands/services, presets/capabilities
├── .github/workflows/           # Split release workflows: Windows default, full updater manual
├── docs/superpowers/            # Specs and implementation plans from previous agentic work
├── assets/ icons/               # README demo and app icon source assets
└── package.json                 # npm scripts; Tauri scripts proxy into src-tauri
```

## WHERE TO LOOK

| Task | Location | Notes |
|------|----------|-------|
| Add or wire a page | `src/App.tsx`, `src/components/Layout/MainLayout.tsx`, `src/store/uiStore.ts` | No React Router; `currentPage` selects page. |
| Frontend IPC types/calls | `src/services/tauri.ts` | Keep invoke names aligned with Rust commands and `main.rs`. |
| Model/provider status UI | `src/components/Models/ProviderStatus.tsx` | Large file; uses canonical `provider.id` for mutations. |
| Provider config UI | `src/components/Providers/ProviderList.tsx`, `ApiKeyModal.tsx`, `CustomProviderModal.tsx` | Config tab, icon cache, API key/base URL flows. |
| Shared preload/cache state | `src/store/preloadStore.ts` | Starts config load first, then background model/version refresh. |
| Config file read/write | `src-tauri/src/services/config_service.rs`, `path_service.rs` | JSON/JSONC, backups, atomic writes, config path candidates. |
| Provider classification/auth/cache | `src-tauri/src/services/provider_service.rs`, `provider_store.rs` | Built-in truth is catalog metadata plus local preset fallback. |
| Model list retrieval | `src-tauri/src/services/model_service.rs` | Cache first, `opencode models` validation later, models.dev details cached. |
| Tauri command registration | `src-tauri/src/main.rs`, `src-tauri/src/commands/*` | Every IPC command must be in `generate_handler!`. |
| Release/updater workflows | `.github/workflows/*.yml`, `src-tauri/tauri.conf.json` | Windows release is automatic on `v*`; full updater is manual. |
| Specs/plans | `docs/superpowers/specs`, `docs/superpowers/plans` | Preserve as design history, not runtime docs. |

## CODE MAP

| Symbol | Type | Location | Role |
|--------|------|----------|------|
| `App` | React component | `src/App.tsx` | Page switch on `useUIStore().currentPage`. |
| `usePreloadStore` | Zustand store | `src/store/preloadStore.ts` | Config, model, provider, version preload cache and refresh locks. |
| `tauriService` plus named wrappers | TS IPC facade | `src/services/tauri.ts` | Single frontend surface for Tauri invoke calls. |
| `main` | Tauri entry | `src-tauri/src/main.rs` | Plugins, tray setup, command handler list, macOS close/reopen behavior. |
| `config_service` | Rust service | `src-tauri/src/services/config_service.rs` | Read/write OMO config while preserving unknown JSON fields. |
| `provider_service` | Rust service | `src-tauri/src/services/provider_service.rs` | Provider status, auth config, built-in/custom classification. |
| `provider_store` | Rust store helpers | `src-tauri/src/services/provider_store.rs` | Raw auth, opencode config, cache, preset file access. |
| `model_service` | Rust service | `src-tauri/src/services/model_service.rs` | Cached/verified model lists and models.dev metadata. |
| `preset_service` | Rust service | `src-tauri/src/services/preset_service.rs` | Preset files, active preset, case-only rename handling. |
| `import_export_service` | Rust service | `src-tauri/src/services/import_export_service.rs` | Backup history, import validation, restore/export. |
| `version_service` | Rust service | `src-tauri/src/services/version_service.rs` | OpenCode/OMO install detection and update hints. |
| `tray::setup_tray` | Rust tray setup | `src-tauri/src/tray.rs` | Tray menu model switching and macOS window behavior. |

## CONVENTIONS

- TypeScript is strict: `noUnusedLocals`, `noUnusedParameters`, `noFallthroughCasesInSwitch` are enabled in `tsconfig.json`.
- Frontend state uses Zustand. Persist keys currently include `omo-ui-storage`, `omo-preload-storage`, `omo-config-storage`, and `omo-switch-language`.
- Frontend should call `src/services/tauri.ts` wrappers, not raw `invoke`, except when extending the wrapper itself.
- Rust services return `Result<T, String>` with user-facing Chinese errors in many paths; match nearby wording.
- OpenCode config writes use `serde_json::Value` to preserve unknown fields. Do not replace config objects with narrow structs.
- Tests that mutate `HOME`, `USERPROFILE`, `XDG_*`, or `OPENCODE_*` use `serial_test::serial` and restore env vars.
- Locale keys are duplicated across 5 JSON files. Add/update all locale files for user-visible strings.
- Git commit style in this repo is Chinese Conventional Commits, e.g. `feat: 修正供应商内置判定`.

## ANTI-PATTERNS (THIS PROJECT)

- Do not remove `#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]` from `src-tauri/src/main.rs`.
- Do not write OMO-private fields into OpenCode `provider.*`; OpenCode schema rejects unknown provider fields.
- Do not use `verified-provider-models.json` or successful `opencode models` output as built-in provider truth; custom config providers can appear there.
- Do not block first render on `opencode models`, models.dev, or version checks. Use cache first and background refresh.
- Do not assume provider display `name` is the provider key. Mutation paths should use canonical `provider.id`.
- Do not edit only one locale file for visible copy changes.

## COMMANDS

```bash
npm run dev
npm run tauri:dev
node .\node_modules\typescript\bin\tsc --noEmit
npm --script-shell=pwsh run build
cargo check --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml provider_service
cargo test --manifest-path src-tauri/Cargo.toml test_add_custom_model
```

## NOTES

- `npm run build` may fail in this Windows/OpenCode shell if `tsc` is not resolved; `npm --script-shell=pwsh run build` has been the reliable local build command.
- App versions are split: `package.json` and `src-tauri/tauri.conf.json` are `1.2.12`, while `src-tauri/Cargo.toml` is `1.2.11` at this commit.
- `src-tauri/src/main.rs` currently emits pre-existing warnings for unused macOS-only names on non-macOS targets.
- Release tags with a hyphen are treated as prereleases by both workflows.
