# FRONTEND KNOWLEDGE BASE

## OVERVIEW

React/Vite frontend. Navigation is store-driven, IPC is centralized in `src/services/tauri.ts`, and most expensive data loads run through `usePreloadStore`.

## STRUCTURE

```text
src/
├── components/          # Domain components plus common UI primitives
├── pages/               # Page shells selected by App.tsx
├── store/               # Zustand stores and persisted preload caches
├── services/tauri.ts    # Tauri IPC wrappers and shared TS types
├── locales/             # zh-CN, zh-TW, en, ja, ko translation JSON
├── utils/               # Tauri environment helpers and model variant rules
└── hooks/               # Cross-page hooks, currently config-change detection
```

## WHERE TO LOOK

| Task | Location | Notes |
|------|----------|-------|
| Add page/navigation | `App.tsx`, `components/Layout/MainLayout.tsx`, `store/uiStore.ts` | No router. Add page key and sidebar copy together. |
| Add IPC call | `services/tauri.ts` | Define TS types here, then call from components/stores. |
| Initial data flow | `store/preloadStore.ts` | Config first, model/version refresh in background after 1200 ms. |
| Toasts/modals/buttons | `components/common/*` | Use existing primitives before adding UI helpers. |
| Model provider status | `components/Models/ProviderStatus.tsx` | Uses `models.grouped`, `models.providers`, and `getProviderStatus()`. |
| Provider configuration | `pages/ProviderPage.tsx`, `components/Providers/*` | Status/config tabs share `refreshModels`. |
| Model variant rules | `utils/modelCapabilities.ts` | GPT-5/OpenAI reasoning variants are special-cased here. |
| Config change alert | `hooks/useConfigChangeDetection.ts`, `components/ConfigChangeAlert/*` | Snapshot compare/merge flow. |

## CONVENTIONS

- Use semicolon style already present in the target file. This repo mixes semicolon/no-semicolon files.
- Use `usePreloadStore` for config/model/version data that is shared across pages.
- Keep loading/error flags in store state, but persisted store partials reset transient flags to non-loading values.
- User-facing strings go through `react-i18next`; update all 5 locale JSON files for new copy.
- Styling is Tailwind classes plus `cn` from `components/common/cn.ts`; no design-token layer exists.
- Browser QA can force a page by seeding `localStorage['omo-ui-storage']` with `currentPage`.

## ANTI-PATTERNS

- Do not call raw `invoke` from pages/components when a wrapper belongs in `services/tauri.ts`.
- Do not add React Router or URL routing without reworking `App.tsx` and `MainLayout` intentionally.
- Do not block page render on models.dev, version checks, or `opencode models`; use cache-first/background refresh.
- Do not assume `ProviderInfo.name` is stable for writes; use `ProviderInfo.id` for provider mutations.
- Do not add visible text in TSX without adding locale keys.
