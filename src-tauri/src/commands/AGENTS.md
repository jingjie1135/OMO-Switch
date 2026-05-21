# RUST COMMANDS KNOWLEDGE BASE

## OVERVIEW

Tauri IPC surface. Command modules translate frontend calls into service calls, expose serde DTOs, and hold tests for command-specific config mutations.

## WHERE TO LOOK

| Task | Location | Notes |
|------|----------|-------|
| Config commands | `config_commands.rs` | Reads/writes OMO config, agent/category model updates, metadata. |
| Config-cache commands | `config_cache_commands.rs` | Snapshot compare, merge, accept external changes. |
| Model commands | `model_commands.rs` | Available/verified models and models.dev metadata. |
| Provider commands | `provider_commands.rs` | Provider status, auth, custom provider/model, model limits, icon cache. |
| Preset commands | `preset_commands.rs` | Thin wrapper over preset service. |
| Import/export commands | `import_export_commands.rs` | Export/import/restore/backup limits. |
| i18n/version commands | `i18n_commands.rs`, `version_commands.rs` | Locale and installed-version surfaces. |

## CONVENTIONS

- Add `#[tauri::command]` on exported IPC functions and register them in `src-tauri/src/main.rs`.
- Add or update the matching TypeScript wrapper in `src/services/tauri.ts` in the same change.
- Public command DTOs use `serde::{Serialize, Deserialize}` and keep frontend shape stable.
- Expensive blocking reads that can affect startup may be wrapped in `tokio::task::spawn_blocking`, as `read_omo_config` does.
- Command tests that mutate HOME/USERPROFILE must be serial and restore env vars before assertions that can panic later.

## ANTI-PATTERNS

- Do not leave a command only in `commands/mod.rs`; it must also be in `generate_handler!`.
- Do not silently create missing providers in `add_custom_model`; the provider must already exist.
- Do not replace existing model objects when updating model limits; preserve sibling fields and set only `limit`.
- Do not fetch provider icons without respecting cache and timeout behavior in `provider_commands.rs`.
- Do not put broad service business rules in command modules unless they are strictly IPC-boundary validation.
