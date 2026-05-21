# RUST SERVICES KNOWLEDGE BASE

## OVERVIEW

Service modules own durable logic: path resolution, OpenCode config preservation, auth/cache IO, model/provider aggregation, presets, import/export, and version detection.

## WHERE TO LOOK

| Task | Location | Notes |
|------|----------|-------|
| Path rules | `path_service.rs` | HOME/USERPROFILE/XDG/OPENCODE env precedence. |
| Config read/write | `config_service.rs` | `oh-my-openagent` primary, legacy `oh-my-opencode`, JSONC accepted. |
| Provider raw storage | `provider_store.rs` | `auth.json`, `opencode.json`, provider model caches, icon cache. |
| Provider status/classification | `provider_service.rs` | Built-in set: catalog metadata plus `presets/providers.json` fallback. |
| Model list/cache | `model_service.rs` | `provider-models.json`, `verified-provider-models.json`, custom merge. |
| Presets | `preset_service.rs` | Metadata wrapper, active preset, case-only rename support. |
| Import/export | `import_export_service.rs` | Managed backups and max backup history setting. |
| Version checks | `version_service.rs` | Detects OpenCode/OMO binaries, packages, config plugins, npm latest. |
| Config snapshots | `config_cache_service.rs` | External-change detection and merge helpers. |

## CONVENTIONS

- `path_service` is the source of truth for config/cache/data directories. Add path variants there first.
- `config_service::write_string_atomically` writes temp files then renames; reuse for durable config writes.
- `config_service::write_omo_config` creates `.bak` before writing and preserves unknown fields.
- `model_service` returns cache data immediately and treats `opencode models` as validation, not the only source.
- `provider_service` treats providers discovered only from cache/auth as known built-ins unless user config proves custom shape and the ID is absent from catalog truth.
- Tests commonly create temp HOME trees under `std::env::temp_dir()` and restore env vars manually.

## ANTI-PATTERNS

- Do not use `verified-provider-models.json` as built-in provider truth.
- Do not collapse `provider-models.json` object entries to strings before catalog metadata has been inspected.
- Do not narrow OpenCode config to typed structs if the write path must preserve unknown fields.
- Do not call `dirs::home_dir` directly for OpenCode paths when `path_service::user_home_dir` handles Windows precedence.
- Do not make models.dev or npm latest failures fatal for ordinary UI status paths.
