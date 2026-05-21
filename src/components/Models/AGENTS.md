# MODELS COMPONENTS KNOWLEDGE BASE

## OVERVIEW

Model browsing, provider status, custom model entry, model limit editing, and applying a model to agents/categories live here.

## WHERE TO LOOK

| Task | Location | Notes |
|------|----------|-------|
| Provider status cards | `ProviderStatus.tsx` | Large coupled file; split mental model into status load, grouping, cards, modals. |
| Manual custom model entry | `AddModelModal.tsx` | Custom providers only; trims ID and checks empty/duplicate before save. |
| Apply model to config | `ApplyModelModal.tsx` | Uses `usePreloadStore` and `usePresetStore`; variant handling matters. |
| Edit custom model limits | `ModelLimitModal.tsx` | Writes `provider.<id>.models.<model>.limit`. |
| Browse model catalog | `ModelBrowser.tsx` | Uses preload model data and models.dev metadata. |
| Model capability options | `../../utils/modelCapabilities.ts` | OpenAI reasoning models do not use legacy `max` the same way. |

## CONVENTIONS

- Provider mutations must pass canonical provider IDs, not display names.
- `ProviderStatus.tsx` reads custom models from backend and caches them in `omo-custom-models-cache-v1` for fallback display.
- Built-in providers do not show Add Model. Custom providers show manual entry only.
- Custom model delete and limit edit actions render only for model IDs returned by `getCustomModels()`.
- Model groups are normalized by model count descending, then provider/model names with locale-aware sorting.
- Applying a model should refresh or update the active preset state when the applied target changes a preset-backed config.

## ANTI-PATTERNS

- Do not reintroduce cross-provider model picking for custom provider Add Model.
- Do not derive `isBuiltin` from model availability alone; backend `ProviderInfo.is_builtin` is the source.
- Do not remove local store updates after add/remove unless you replace them with an equivalent refresh path.
- Do not write model limit metadata outside the OpenCode-supported `limit` object.
- Do not add Add Model buttons for providers where `provider.isBuiltin` is true.
