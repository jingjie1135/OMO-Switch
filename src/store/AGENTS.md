# FRONTEND STORES KNOWLEDGE BASE

## OVERVIEW

Zustand state layer. Stores hold UI navigation, cached OMO config, model/provider/version preload data, presets, update state, and older config/model state.

## WHERE TO LOOK

| Task | Location | Notes |
|------|----------|-------|
| First-load data flow | `preloadStore.ts` | Loads config first, then background-refreshes models and versions. |
| Agent/category optimistic updates | `preloadStore.ts` | `variant: 'none'` removes the variant field locally. |
| Persist current page/sidebar | `uiStore.ts` | `omo-ui-storage`; `currentPage` drives `App.tsx`. |
| Active preset state | `presetStore.ts` | Coordinates with backend `get_active_preset` / `set_active_preset`. |
| App updater state | `updaterStore.ts` | Wraps Tauri updater plugin status, progress, and modal open state. |
| Legacy config/model stores | `configStore.ts`, `modelStore.ts` | Still present; verify consumers before extending. |

## CONVENTIONS

- Stores use `create` from Zustand; persisted stores use `persist` plus `createJSONStorage(() => localStorage)`.
- `preloadStore` persists data snapshots only. `loading`, `error`, `validating`, and private refresh locks are reset by `partialize`.
- `_modelsRefreshing`, `_omoConfigRefreshing`, and `_versionsRefreshing` are request locks; avoid replacing them with component-local guards.
- `refreshModels()` returns cache data first, then updates validation status when `getAvailableModelsWithStatus()` resolves.
- `startPreload()` ensures the `default` preset exists but does not let preset initialization failure block the first render.
- Use `useShallow` or selectors for large slices to avoid broad rerenders in page components.

## ANTI-PATTERNS

- Do not persist transient loading/error state into localStorage.
- Do not block `startPreload()` on model validation, models.dev, or version checks.
- Do not call backend writes directly from stores if the page/component must also show a confirmation modal or toast.
- Do not assume `models.grouped` is verified; inspect `models.source`, `fallbackReason`, and `validating` when that distinction matters.
- Do not update only agents when the same UI action can target categories; keep agent/category paths paired.
