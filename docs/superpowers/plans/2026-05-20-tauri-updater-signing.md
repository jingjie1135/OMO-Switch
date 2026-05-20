# Tauri Updater Signing Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Re-enable signed Tauri updater publishing for this fork with a new owner-controlled, password-protected updater keypair.

**Architecture:** The Tauri app stores only the updater public key in `src-tauri/tauri.conf.json`. GitHub Actions receives the private key through Secrets, keeps platform code signing disabled with `--no-sign`, signs updater bundles explicitly, and uploads an aggregated `latest.json` during tagged releases.

**Tech Stack:** Tauri 2, `cargo tauri signer`, GitHub Actions, `tauri-apps/tauri-action@v0`, JSON/YAML/Markdown.

---

### Task 1: Generate updater keypair

**Files:**
- Create outside repo: `~/.tauri/omo-switch.key`
- Create outside repo: `~/.tauri/omo-switch.key.pub`

- [ ] **Step 1: Verify key directory**

Run: `Test-Path -LiteralPath "$HOME\.tauri"`
Expected: `True`; if false, create it with `New-Item -ItemType Directory -Path "$HOME\.tauri"`.

- [ ] **Step 2: Generate keypair**

Run: `cargo tauri signer generate --ci -p "<strong-password>" -w "$HOME\.tauri\omo-switch.key"`
Expected: output includes `Your keypair was generated successfully`, `Private: ...omo-switch.key`, and `Public: ...omo-switch.key.pub`.

- [ ] **Step 3: Read public key**

Run: read `~/.tauri/omo-switch.key.pub`.
Expected: one-line base64 public key string.

### Task 2: Update app updater public key

**Files:**
- Modify: `src-tauri/tauri.conf.json`

- [ ] **Step 1: Replace updater public key**

Set `plugins.updater.pubkey` to the contents of `~/.tauri/omo-switch.key.pub`.

- [ ] **Step 2: Keep endpoint unchanged**

Ensure `plugins.updater.endpoints` still points at `https://github.com/ShellMonster/OMO-Switch/releases/latest/download/latest.json` unless the repository owner changes release location.

- [ ] **Step 3: Verify JSON**

Run: `node -e "JSON.parse(require('fs').readFileSync('src-tauri/tauri.conf.json','utf8')); console.log('ok')"`
Expected: `ok`.

### Task 3: Restore release updater publishing

**Files:**
- Modify: `.github/workflows/release.yml`

-- [ ] **Step 1: Keep platform signing disabled**

Keep `args: ${{ matrix.tauriArgs }} --no-sign` so the workflow does not require Windows or macOS platform code-signing certificates.

- [ ] **Step 2: Include updater JSON**

Change `includeUpdaterJson: false` to `includeUpdaterJson: true`.

- [ ] **Step 3: Restore updater-only signing steps**

Add macOS and Windows steps after `tauri-apps/tauri-action@v0` that require `TAURI_SIGNING_PRIVATE_KEY` and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`, run `npm run -s tauri -- signer sign -p ...` on the generated updater bundle, and upload the matching `.sig` asset.

- [ ] **Step 4: Restore aggregated latest.json job**

Add `publish-updater-json` after the build matrix. It fetches release assets, pairs bundles with uploaded `.sig` files, writes `latest.json`, and uploads it back to the release.

### Task 4: Update updater documentation

**Files:**
- Modify: `README.md`

- [ ] **Step 1: Mention private key storage**

Document that the generated private key remains local and the public key is committed.

- [ ] **Step 2: Document GitHub Secrets**

Document `TAURI_SIGNING_PRIVATE_KEY` as the contents of `~/.tauri/omo-switch.key`; document `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` as the password used when generating the key.

- [ ] **Step 3: Document test tag validation**

Add commands for pushing a test tag and verifying Release assets include `latest.json`.

### Task 5: Verify behavior

**Files:**
- Verify: `src-tauri/tauri.conf.json`
- Verify: `.github/workflows/release.yml`
- Verify: `README.md`

- [ ] **Step 1: Check key references**

Run: `rg -n "TAURI_SIGNING_PRIVATE_KEY|includeUpdaterJson|--no-sign|latest\.json|pubkey" ".github/workflows/release.yml" "src-tauri/tauri.conf.json" "README.md"`
Expected: signing env exists, `includeUpdaterJson: true`, `--no-sign` remains for platform code signing, updater endpoint and docs mention `latest.json`.

- [ ] **Step 2: Run Rust checks**

Run: `cargo check`.
Expected: exit 0.

- [ ] **Step 3: Run version scanner tests**

Run: `cargo test services::version_service::tests -- --nocapture`.
Expected: all version scanner tests pass.

- [ ] **Step 4: Exercise signer surface**

Run: `cargo tauri signer sign --help`.
Expected: signer CLI prints help, proving the updater signing tool is available locally.

- [ ] **Step 5: Report secret copy command**

Report how to retrieve the private key value without committing it: read `~/.tauri/omo-switch.key` locally and paste it into the GitHub Secret.
