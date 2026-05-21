# Custom Provider Classification and Model Entry Design

## Problem

The Providers configuration tab currently labels many OpenCode Desktop built-in providers as custom providers. Examples include `302ai`, `abacus`, `alibaba`, `amazon-bedrock`, `cohere`, `mistral`, `xai`, and `zhipuai`.

The current backend determines `ProviderInfo.is_builtin` by checking whether a provider exists in OMO-Switch's bundled `src-tauri/presets/providers.json`. That file only contains a small local preset list, so any OpenCode provider that is not in this local file is incorrectly reported as custom.

The add-model flow has a related issue: adding a model to a custom provider currently shows models from other providers and lets the user choose one. For a user-defined provider in OpenCode config, the expected behavior is to type a custom model ID directly.

## Definitions

- **Built-in provider**: A provider that comes from OpenCode or OpenCode Desktop's provider catalog. It should be treated as built-in even if OMO-Switch does not have local metadata for it.
- **Custom provider**: A provider entry under the user's OpenCode config `provider` object that is not present in the OpenCode built-in provider catalog. It may be created through OMO-Switch's “Add Custom Provider” flow or manually added directly to OpenCode config.
- **Custom model**: A model ID manually added under a custom provider's `models` object.

This means provider IDs discovered from OpenCode caches, model lists, auth data, or config should not automatically become custom. A configured provider becomes custom only when it is user-defined in `opencode.json.provider` and absent from the OpenCode built-in provider catalog.

## Goals

1. Stop labeling OpenCode Desktop built-in providers as custom.
2. Make user-defined non-catalog OpenCode config providers the only providers shown as custom.
3. Change the add-model flow for custom providers from “select an existing provider model” to “type a custom model ID”.
4. Preserve existing OpenCode-compatible provider config fields.
5. Preserve existing user-defined custom providers created by older OMO-Switch builds or by direct OpenCode config edits.

## Non-Goals

- Do not attempt to maintain a full static copy of OpenCode's provider catalog in this change.
- Do not add blocking network fetching to the Providers configuration tab.
- Do not write OMO-private fields into OpenCode provider config. The published OpenCode config schema rejects unknown provider fields.
- Do not add model metadata editing beyond the existing limit editor.

## Classification Design

### Config shape

When OMO-Switch creates a custom provider through `add_custom_provider`, it will write only OpenCode-supported provider config fields:

```json
{
  "provider": {
    "my-provider": {
      "npm": "@ai-sdk/openai-compatible",
      "options": {
        "baseURL": "https://example.com/v1"
      },
      "models": {}
    }
  }
}
```

No `omo_custom` marker will be written to `opencode.json`. Providers manually added directly to `opencode.json.provider` must be classified by comparing their provider ID against the built-in provider ID set.

### Backend classification

`ProviderInfo.is_builtin` will be derived from custom-provider detection rather than local preset membership:

- Build a built-in provider ID set from OpenCode's provider catalog source. The practical sources, in priority order, are:
  1. a successful `opencode models` / model verification result grouped by provider;
  2. the raw OpenCode provider-model cache loaded by `provider_store::read_provider_models()`;
  3. OMO-Switch's bundled `src-tauri/presets/providers.json` as metadata fallback only.
- If a provider ID is present in the built-in provider ID set, it is built-in: `is_builtin = true`.
- If a provider ID is present under `opencode.json.provider` and absent from the built-in provider ID set, it is custom: `is_builtin = false`.
- Otherwise, providers discovered only from caches or auth data are treated as built-in/known, not custom.

The local `providers.json` remains useful for metadata such as display name, npm package, and website URL, but it is no longer the source of truth for built-in classification.

### Current observed custom providers

On the current machine, `G:\Users\Administrator\.config\opencode\opencode.json` contains two configured providers:

- `mirror0425`
- `ngle`

Both use `@ai-sdk/openai-compatible`, both have a base URL, both define custom model entries, and neither is present in the checked OpenCode/models.dev provider catalog. Under this design, both are custom providers.

### Legacy compatibility

Older OMO-Switch versions and direct OpenCode edits can create custom providers using only standard OpenCode provider fields. To avoid reclassifying those providers as built-in, the backend will treat a provider as custom when all of these are true:

1. The provider ID is present under `opencode.json.provider`.
2. The provider ID is absent from the OpenCode built-in provider ID set.
3. The provider config has a custom-provider shape, such as `npm: "@ai-sdk/openai-compatible"`, a non-empty `options.baseURL`/`options.baseUrl`, or explicit `models` entries.

This preserves typical user-defined providers while allowing OpenCode built-ins from provider/model caches to default to built-in. The catalog membership check prevents configured built-ins from being mislabeled as custom only because they have auth, base URL overrides, or model metadata overrides.

## Provider Configuration UI Design

The Providers configuration tab will continue to use `ProviderInfo.is_builtin` from the backend.

- Built-in providers show the built-in tag.
- User-defined non-catalog config providers show the custom tag.
- The “unconfigured custom providers” section should no longer imply that OpenCode catalog providers are custom. Since custom providers are user-defined config entries, there is no meaningful preconfigured list of unconfigured custom providers; this section should be hidden when empty.

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
3. Rust builds the OpenCode built-in provider ID set from the available model verification/cache data and local metadata fallback.
4. For each provider ID, Rust checks whether it is configured under `opencode.json.provider` and absent from the built-in provider ID set.
5. Rust returns `ProviderInfo.is_builtin` based on that custom-provider check.
6. Provider configuration UI uses `is_builtin` for tags and grouping.
7. Provider status UI uses `is_builtin` to decide whether the provider can manually add custom models.

## Files Expected to Change

- `src-tauri/src/services/provider_service.rs`
  - Add catalog-based custom-provider detection.
  - Update `add_custom_provider()` to initialize `models: {}` while writing only OpenCode-supported provider fields.
  - Add regression tests for OpenCode built-ins not in local presets, direct OpenCode custom providers, and OMO-created custom providers.
- `src-tauri/src/services/provider_store.rs`
  - Expose or reuse provider/model cache provider IDs as an input to built-in provider detection.
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

- Provider ID from OpenCode provider/model cache that is absent from `providers.json` is treated as built-in.
- Provider ID present only under `opencode.json.provider`, absent from the OpenCode built-in provider ID set, and shaped like an OpenAI-compatible/baseURL/models override is treated as custom.
- Provider created by `add_custom_provider()` includes `models: {}`, does not include unknown private config fields, and returns `is_builtin = false`.
- OpenCode provider catalog/cache membership wins over config shape, so built-ins with auth/base URL/model overrides still return `is_builtin = true`.

Frontend verification:

- The Providers configuration tab shows OpenCode catalog providers as built-in instead of custom.
- A direct OpenCode config custom provider such as `mirror0425` or `ngle` shows as custom.
- A provider created through OMO-Switch shows as custom.
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
- Add or inspect a user-defined non-catalog provider and confirm it is labeled custom.
- Use the custom provider add-model button to enter a model ID and confirm the model appears under that provider.
