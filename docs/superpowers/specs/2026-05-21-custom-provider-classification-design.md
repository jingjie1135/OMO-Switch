# Custom Provider Classification and Model Entry Design

## Problem

The Providers configuration tab currently labels many OpenCode Desktop built-in providers as custom providers. Examples include `302ai`, `abacus`, `alibaba`, `amazon-bedrock`, `cohere`, `mistral`, `xai`, and `zhipuai`.

The current backend determines `ProviderInfo.is_builtin` by checking whether a provider exists in OMO-Switch's bundled `src-tauri/presets/providers.json`. That file only contains a small local preset list, so any OpenCode provider that is not in this local file is incorrectly reported as custom.

The add-model flow has a related issue: adding a model to a custom provider currently shows models from other providers and lets the user choose one. For a user-created provider, the expected behavior is to type a custom model ID directly.

## Definitions

- **Built-in provider**: A provider that comes from OpenCode or OpenCode Desktop's provider catalog. It should be treated as built-in even if OMO-Switch does not have local metadata for it.
- **Custom provider**: A provider manually created by the user through OMO-Switch's “Add Custom Provider” flow.
- **Custom model**: A model ID manually added under a custom provider's `models` object.

This means provider IDs discovered from OpenCode caches, model lists, auth data, or config should not automatically become custom. Only providers explicitly created by OMO-Switch as user custom providers should be custom.

## Goals

1. Stop labeling OpenCode Desktop built-in providers as custom.
2. Make user-created providers the only providers shown as custom.
3. Change the add-model flow for custom providers from “select an existing provider model” to “type a custom model ID”.
4. Preserve existing OpenCode-compatible provider config fields.
5. Preserve likely existing user-created custom providers created by older OMO-Switch builds.

## Non-Goals

- Do not attempt to maintain a full static copy of OpenCode's provider catalog in this change.
- Do not add dynamic network fetching to the Providers configuration tab.
- Do not change OpenCode's provider config schema beyond an OMO-owned marker for user-created providers.
- Do not add model metadata editing beyond the existing limit editor.

## Classification Design

### New marker

When OMO-Switch creates a custom provider through `add_custom_provider`, it will write an OMO-owned marker into that provider object:

```json
{
  "provider": {
    "my-provider": {
      "npm": "@ai-sdk/openai-compatible",
      "options": {
        "baseURL": "https://example.com/v1"
      },
      "models": {},
      "omo_custom": true
    }
  }
}
```

`omo_custom` is only used by OMO-Switch to distinguish user-created providers from OpenCode built-ins. It is not used for request routing.

### Backend classification

`ProviderInfo.is_builtin` will be derived from custom-provider detection rather than local preset membership:

- If a provider has `omo_custom: true`, it is custom: `is_builtin = false`.
- If a provider matches legacy custom-provider heuristics and is not known by OpenCode's provider/model cache, it is custom: `is_builtin = false`.
- Otherwise, it is built-in: `is_builtin = true`.

The local `providers.json` remains useful for metadata such as display name, npm package, and website URL, but it is no longer the source of truth for built-in classification.

### Legacy compatibility

Older OMO-Switch versions created custom providers without `omo_custom`. To avoid reclassifying those providers as built-in, the backend will treat a provider as legacy custom when all of these are true:

1. The provider ID is not present in `src-tauri/presets/providers.json`.
2. The provider ID is not present in the raw OpenCode provider-model cache loaded by `provider_store::read_provider_models()`.
3. The provider config uses `npm: "@ai-sdk/openai-compatible"`.
4. The provider config has a non-empty `options.baseURL` or `options.baseUrl`.
5. The provider has an auth entry.

This heuristic preserves typical older user-created providers while allowing OpenCode built-ins from provider/model caches to default to built-in. The OpenCode provider-model cache condition prevents configured built-ins from being mislabeled as legacy custom only because they have auth or a base URL.

## Provider Configuration UI Design

The Providers configuration tab will continue to use `ProviderInfo.is_builtin` from the backend.

- Built-in providers show the built-in tag.
- User-created providers show the custom tag.
- The “unconfigured custom providers” section should no longer imply that OpenCode catalog providers are custom. If the section has no real user-created custom providers, it should be empty or hidden according to existing ProviderList behavior.

## Custom Provider Model Entry Design

Only custom providers may show the “Add Model” action in the Provider status tab.

The add-model modal will become a manual entry form:

- Title: add custom model.
- Body: explain that the model ID should match the model name exposed by the custom provider.
- Input: model ID text field.
- Validation:
  - Trim whitespace.
  - Require a non-empty model ID.
  - Prevent adding a duplicate model ID already present under the same provider.
- Save behavior:
  - Call existing `addCustomModel(providerId, modelId)`.
  - Backend writes `provider.<providerId>.models.<modelId> = {}` if missing.
  - Refresh model data after success.

The modal will no longer receive or browse `providerModels` from other providers.

## Data Flow

1. `ProviderPage` calls `getProviderStatus()`.
2. Rust `provider_service::get_provider_status()` aggregates provider IDs from presets, provider model cache, connected providers, auth data, and OpenCode config.
3. For each provider ID, Rust checks whether it is user custom via `omo_custom` or the legacy heuristic.
4. Rust returns `ProviderInfo.is_builtin` based on that custom-provider check.
5. Provider configuration UI uses `is_builtin` for tags and grouping.
6. Provider status UI uses `is_builtin` to decide whether the provider can manually add custom models.

## Files Expected to Change

- `src-tauri/src/services/provider_service.rs`
  - Add custom-provider detection.
  - Update `add_custom_provider()` to write `omo_custom: true` and initialize `models: {}`.
  - Add regression tests for OpenCode built-ins not in local presets and OMO-created custom providers.
- `src/components/Models/ProviderStatus.tsx`
  - Pass provider custom/built-in status into provider cards.
  - Show add-model action only for custom providers.
- `src/components/Models/AddModelModal.tsx`
  - Replace existing-model picker with manual model ID input.
  - Remove dependency on provider-wide model lists.
- `src/services/tauri.ts`
  - No backend command change is expected, but comments/types may be updated if needed.
- Locale files under `src/locales/`
  - Update add-model copy from selection wording to manual-entry wording.

## Testing and Verification

Backend tests:

- Provider ID from config/cache that is absent from `providers.json` and lacks `omo_custom` is treated as built-in.
- Provider created by `add_custom_provider()` includes `omo_custom: true`, `models: {}`, and returns `is_builtin = false`.
- Legacy custom provider heuristic returns `is_builtin = false` for an older OpenAI-compatible custom provider with base URL and auth when the provider is absent from the raw OpenCode provider-model cache.
- OpenCode provider cache membership wins over the legacy heuristic for an unmarked configured provider, so built-ins with auth/base URL still return `is_builtin = true`.

Frontend verification:

- The Providers configuration tab shows OpenCode catalog providers as built-in instead of custom.
- A user-created provider shows as custom.
- Built-in providers do not expose manual add-model UI.
- Custom providers expose manual add-model UI.
- Entering a new model ID saves it and refreshes the provider model list.
- Empty and duplicate model IDs are blocked before save.

Build verification:

- `cargo check --manifest-path src-tauri/Cargo.toml`
- Relevant Rust tests for provider service/commands.
- `node .\\node_modules\\typescript\\bin\\tsc --noEmit`
- `node .\\node_modules\\vite\\bin\\vite.js build`

Manual QA surface:

- Open the Providers page in a browser/Tauri surface.
- Confirm a provider from the OpenCode built-in list but absent from local presets is labeled built-in.
- Add or inspect a user-created provider and confirm it is labeled custom.
- Use the custom provider add-model button to enter a model ID and confirm the model appears under that provider.
