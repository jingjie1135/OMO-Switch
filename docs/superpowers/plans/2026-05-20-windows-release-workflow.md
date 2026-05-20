# Windows Release Workflow Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make Windows x64 installer publishing the default automatic release path while keeping the full updater workflow available only on demand.

**Architecture:** The existing `.github/workflows/release.yml` becomes `Full Updater Release` and only exposes `workflow_dispatch`. A new `.github/workflows/windows-release.yml` is named `Release`, triggers on `v*` tags, builds only the Windows NSIS x64 installer with platform signing skipped, and does not create updater signatures or `latest.json`.

**Tech Stack:** GitHub Actions, Tauri 2, `tauri-apps/tauri-action@v0`, Windows NSIS bundling, npm/Rust.

---

### Task 1: Make full updater workflow manual-only

**Files:**
- Modify: `.github/workflows/release.yml`

- [ ] **Step 1: Rename the workflow**

Change the first line from:

```yaml
name: Release
```

to:

```yaml
name: Full Updater Release
```

- [ ] **Step 2: Remove automatic tag trigger**

Change the trigger block from:

```yaml
on:
  push:
    tags:
      - 'v*'
  workflow_dispatch:
```

to:

```yaml
on:
  workflow_dispatch:
```

- [ ] **Step 3: Keep full updater behavior unchanged**

Leave the matrix, updater signing steps, `includeUpdaterJson: true`, and `publish-updater-json` job unchanged.

### Task 2: Add automatic Windows x64 release workflow

**Files:**
- Create: `.github/workflows/windows-release.yml`

- [ ] **Step 1: Create workflow with tag and manual triggers**

Add a workflow named `Release` with:

```yaml
on:
  push:
    tags:
      - 'v*'
  workflow_dispatch:
```

- [ ] **Step 2: Add Windows-only build job**

Use one job on `windows-latest`. Install Node.js 20, Rust stable, cache Rust, run `npm ci`, compute `BUILD_TIME`, generate release notes, then call `tauri-apps/tauri-action@v0`.

- [ ] **Step 3: Build only NSIS x64 installer**

Set action args to:

```yaml
args: --bundles nsis --no-sign
```

Set:

```yaml
includeUpdaterJson: false
includeRelease: true
```

Do not add `TAURI_SIGNING_PRIVATE_KEY`, `.sig` upload steps, macOS jobs, or `latest.json` jobs.

- [ ] **Step 4: Mark test tags as prerelease**

Use:

```yaml
prerelease: ${{ contains(github.ref_name, '-') }}
```

### Task 3: Verify workflow split

**Files:**
- Verify: `.github/workflows/release.yml`
- Verify: `.github/workflows/windows-release.yml`

- [ ] **Step 1: Search workflow triggers and updater references**

Run:

```bash
rg -n "name:|push:|tags:|workflow_dispatch|includeUpdaterJson|TAURI_SIGNING|latest\.json|macOS|Windows|--bundles nsis|--no-sign" .github/workflows
```

Expected:
- `release.yml` is `Full Updater Release`, manual-only, still has updater signing/latest.json.
- `windows-release.yml` is `Release`, has `push.tags`, Windows-only, no updater signing/latest.json.

- [ ] **Step 2: Run local project checks**

Run:

```bash
cargo check
cargo test services::version_service::tests -- --nocapture
npm run build
```

Expected: all exit 0; warnings are acceptable if pre-existing.

### Task 4: Commit and GitHub validation

**Files:**
- Commit: `.github/workflows/release.yml`
- Commit: `.github/workflows/windows-release.yml`
- Commit: `docs/superpowers/plans/2026-05-20-windows-release-workflow.md`

- [ ] **Step 1: Commit workflow split**

Use message:

```bash
ci: 拆分 Windows 默认发布工作流
```

- [ ] **Step 2: Push main**

Run:

```bash
git push origin main
```

- [ ] **Step 3: Trigger test tag**

Create and push a new test tag, e.g.:

```bash
git tag v1.2.13-test.2
git push origin v1.2.13-test.2
```

- [ ] **Step 4: Verify release assets**

Use `gh run watch` and `gh release view v1.2.13-test.2`.
Expected: the new `Release` workflow succeeds and the test prerelease contains Windows x64 setup installer artifacts only, with no `latest.json`, no `.sig`, and no macOS assets.
