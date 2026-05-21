# RELEASE WORKFLOWS KNOWLEDGE BASE

## OVERVIEW

Two release paths exist. `windows-release.yml` is the automatic `v*` tag path for Windows x64 NSIS installers; `release.yml` is the manual full updater release with macOS, Windows, signatures, and aggregated `latest.json`.

## WHERE TO LOOK

| Task | Location | Notes |
|------|----------|-------|
| Default tag release | `windows-release.yml` | Workflow name `Release`, triggers on `v*` and manual dispatch. |
| Full updater release | `release.yml` | Workflow name `Full Updater Release`, manual dispatch only. |
| Updater public key | `../../src-tauri/tauri.conf.json` | Committed `plugins.updater.pubkey`. |
| Updater docs/plans | `../../README.md`, `../../docs/superpowers/plans/2026-05-20-tauri-updater-signing.md` | Secret setup and signing flow. |

## CONVENTIONS

- Both workflows use Node 20, stable Rust, `npm ci`, and `tauri-apps/tauri-action@v0`.
- Platform code signing stays disabled with `--no-sign`; updater bundle signing is separate.
- Test/prerelease tags contain `-`; workflows set `prerelease: ${{ contains(github.ref_name, '-') }}`.
- Full updater release requires `TAURI_SIGNING_PRIVATE_KEY` and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`.
- `release.yml` uploads `.sig` assets then generates `latest.json` containing darwin-aarch64, darwin-x86_64, and windows-x86_64.

## ANTI-PATTERNS

- Do not add updater signing secrets to `windows-release.yml`; it intentionally produces installer assets only.
- Do not remove `workflow_dispatch` from the full updater workflow.
- Do not commit the private updater key. Only the public key belongs in `tauri.conf.json`.
- Do not make `latest.json` generation optional in the full updater workflow; missing platforms intentionally fail it.

## COMMANDS

```bash
rg -n "name:|push:|workflow_dispatch|includeUpdaterJson|TAURI_SIGNING|latest\.json|--bundles nsis|--no-sign" .github/workflows
```
