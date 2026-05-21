# TAURI BACKEND KNOWLEDGE BASE

## OVERVIEW

Rust/Tauri backend. `main.rs` registers plugins and IPC commands; command modules expose Tauri surfaces; service modules own file IO, OpenCode config semantics, cache parsing, and release/version logic.

## STRUCTURE

```text
src-tauri/
├── src/main.rs             # Tauri builder, tray setup, invoke_handler list
├── src/commands/           # IPC command functions and command-local DTOs/tests
├── src/services/           # Durable backend logic and filesystem/cache access
├── src/tray.rs             # Tray menu and model switching actions
├── src/i18n.rs             # Rust-side localized messages
├── presets/providers.json  # Local metadata fallback, not full built-in truth
├── capabilities/default.json
└── tauri.conf.json         # App version, updater endpoint/pubkey, bundle settings
```

## WHERE TO LOOK

| Task | Location | Notes |
|------|----------|-------|
| Register IPC | `src/main.rs`, `src/commands/mod.rs`, `src/commands/*.rs` | Add to `generate_handler!` and frontend wrapper. |
| Config paths | `src/services/path_service.rs` | Central source for HOME/USERPROFILE/XDG/OPENCODE paths. |
| OMO config IO | `src/services/config_service.rs` | Candidate filenames, JSONC, backup, atomic write. |
| Provider/auth/cache IO | `src/services/provider_store.rs` | Raw files under opencode config/data/cache dirs. |
| Provider business logic | `src/services/provider_service.rs` | Built-in/custom status, auth writes, custom provider creation. |
| Model list business logic | `src/services/model_service.rs` | Cache-first, `opencode models` validation, models.dev cache. |
| Import/export/backups | `src/services/import_export_service.rs` | Managed backup history and restore guardrails. |
| Release updater | `tauri.conf.json`, `.github/workflows/release.yml` | Public key is committed; private key is GitHub Secret only. |

## CONVENTIONS

- Commands return `Result<_, String>` and should stay thin unless the behavior is purely command-local.
- Services preserve unknown JSON fields by editing `serde_json::Value` objects.
- Config writes should go through existing service/store helpers so backups and atomic writes remain consistent.
- Any test mutating process env must be `#[serial_test::serial]` and restore original env vars.
- Prefer `path_service` over direct `dirs::*` calls for OpenCode/OMO paths.
- Keep Tauri command names snake_case; frontend calls them through camelCase TS wrapper args where existing wrappers already do so.

## ANTI-PATTERNS

- Do not add a `lib.rs` assumption; this app currently uses `main.rs` as the crate entry.
- Do not forget `src-tauri/src/main.rs` when adding a command. Unregistered commands compile but fail at runtime.
- Do not remove the Windows subsystem attribute in `main.rs`; it suppresses the release console window.
- Do not put OMO-private marker fields into OpenCode config provider objects.
- Do not make network calls mandatory for provider/model status. Offline/cache fallback is intentional.

## COMMANDS

```bash
cargo check --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml provider_service
cargo test --manifest-path src-tauri/Cargo.toml provider_commands
cargo tauri signer sign --help
```
