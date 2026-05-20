# Custom Provider Model Limit Metadata Design

## Goal

Add a user-facing way in OMO-Switch to configure OpenCode model limit metadata for custom provider models, so OpenCode can use accurate context, input, and output token limits when calculating automatic context compaction thresholds.

## Scope

This feature covers only the OpenCode model `limit` object for models under custom providers:

```json
{
  "provider": {
    "my-provider": {
      "models": {
        "my-model": {
          "limit": {
            "context": 128000,
            "input": 120000,
            "output": 8192
          }
        }
      }
    }
  }
}
```

The first version does not edit pricing, capabilities, names, descriptions, provider npm package, or other model metadata.

## Current State

- `src-tauri/src/commands/provider_commands.rs` exposes `add_custom_model`, `remove_custom_model`, and `get_custom_models`.
- `add_custom_model` currently creates `provider.<providerId>.models.<modelId> = {}`.
- `src-tauri/src/services/provider_store.rs` reads and writes `opencode.json` / `opencode.jsonc`, preserving existing JSON object fields during normal writes.
- `get_custom_models()` returns only `Record<providerId, modelId[]>`, so the frontend cannot currently inspect per-model config objects.
- `src/components/Models/ProviderStatus.tsx` already marks custom models and renders model-level action buttons, making it the best UI insertion point.
- `src/components/Models/AddModelModal.tsx` is a lightweight model picker and should stay focused on quick model addition.

## User Experience

The provider status model list will add an edit/settings action next to each custom model. The action opens a new `ModelLimitModal` for the selected provider/model.

The modal shows three numeric fields:

- `context`: required, integer greater than `0`.
- `input`: optional, integer greater than `0` when set.
- `output`: required, integer greater than `0`.

Saving updates only that model's `limit` object. Existing non-limit fields in the same model object remain untouched.

The add-model flow remains unchanged: users can add a custom model first, then edit its limit metadata from the model list.

## Backend Design

Add Rust data structures in `src-tauri/src/commands/provider_commands.rs` or a provider service module:

```rust
struct ModelLimit {
    context: u64,
    input: Option<u64>,
    output: u64,
}

struct CustomModelMetadata {
    limit: Option<ModelLimit>,
}
```

Add Tauri commands:

- `get_custom_model_metadata(provider_id: String, model_id: String) -> Result<CustomModelMetadata, String>`
- `update_custom_model_limit(provider_id: String, model_id: String, limit: ModelLimit) -> Result<(), String>`

Command behavior:

- Read the existing OpenCode config through `provider_store::read_opencode_config()`.
- Validate that `provider.<providerId>.models.<modelId>` exists and is an object.
- For `get_custom_model_metadata`, return the parsed `limit` field if present.
- For `update_custom_model_limit`, set only the `limit` property on the model object.
- Preserve all other provider/model fields.
- Write through `provider_store::write_opencode_config()` so the existing backup and path handling remain consistent.

Validation rules:

- `context` and `output` must be positive integers.
- `input`, when provided, must be a positive integer.
- `input` may be greater than, equal to, or less than `context`; OpenCode's schema permits both fields independently, and different providers may report them differently.

## Frontend Design

Add TypeScript types and service calls in `src/services/tauri.ts`:

```ts
export interface ModelLimit {
  context: number;
  input?: number | null;
  output: number;
}

export interface CustomModelMetadata {
  limit?: ModelLimit | null;
}
```

Add functions:

- `getCustomModelMetadata(providerId, modelId)`
- `updateCustomModelLimit(providerId, modelId, limit)`

Add a new component:

- `src/components/Models/ModelLimitModal.tsx`

The modal owns local form state, loads existing metadata on open, validates before save, and reports save errors inline.

Update `src/components/Models/ProviderStatus.tsx`:

- Show the edit/settings action only for `isCustomModel(model)`.
- Keep the existing delete action.
- Open `ModelLimitModal` with the selected provider/model.
- Refresh custom model state after save if needed, but avoid a full model reload unless necessary.

## Data Flow

1. User expands a provider in Provider Status.
2. User clicks edit/settings on a custom model.
3. Frontend calls `get_custom_model_metadata`.
4. Backend reads the model object from OpenCode config and returns `limit`.
5. User edits `context`, `input`, and `output`.
6. Frontend calls `update_custom_model_limit`.
7. Backend validates and writes `provider.<providerId>.models.<modelId>.limit`.
8. OpenCode reads the updated metadata on its next config load and can use the limits for context/compaction behavior.

## Error Handling

- If the provider or model does not exist, return a clear error and keep the modal open.
- If the model entry exists but is not an object, return a clear error instead of replacing it silently.
- If numeric input is invalid, block save in the frontend before calling the backend.
- If backend write fails, show the error in the modal.

## Testing

Backend tests:

- `get_custom_model_metadata` returns an existing full `limit` object.
- `get_custom_model_metadata` returns no limit for an empty model object.
- `update_custom_model_limit` writes `context`, optional `input`, and `output` to the right path.
- Updating `limit` preserves unrelated model fields.
- Updating a missing provider/model returns an error.

Frontend verification:

- Build/typecheck catches service typing and component integration errors.
- Manual QA opens Provider Status, edits a custom model limit, saves, and verifies `opencode.json` contains the expected `limit` object.
- Manual QA tries invalid values and confirms save is blocked or errors are shown.

## Out of Scope

- Editing model pricing or capabilities.
- Editing built-in provider model metadata.
- Inferring default limits from Models.dev.
- Changing OpenCode's compaction algorithm.
- Writing a separate cache file for metadata outside OpenCode config.

## Acceptance Criteria

- Users can edit `context`, `input`, and `output` limits for any custom model.
- The saved config uses `provider.<providerId>.models.<modelId>.limit` with no extra `metadata.model` wrapper.
- Existing custom model fields survive a limit update.
- Existing add/remove custom model behavior remains unchanged.
- Invalid numeric input cannot be saved.
- Backend tests and frontend build pass.
