# PROVIDERS COMPONENTS KNOWLEDGE BASE

## OVERVIEW

Provider configuration tab components. They configure API keys, base URLs, custom providers, provider icon display, and grouping by configured/built-in/custom state.

## WHERE TO LOOK

| Task | Location | Notes |
|------|----------|-------|
| Provider grid/groups | `ProviderList.tsx` | Configured, unconfigured built-in, and non-empty custom groups. |
| Built-in provider auth | `ApiKeyModal.tsx` | Reads existing config snapshot and writes API key/base URL/provider type. |
| Add custom provider | `CustomProviderModal.tsx` | Creates OpenAI-compatible provider with base URL and auth. |
| Provider page orchestration | `../../pages/ProviderPage.tsx` | Owns active status/config tab and refresh behavior. |
| Backend types | `../../services/tauri.ts` | `ProviderInfo`, `ProviderConfigSnapshot`, `ConnectionTestResult`. |

## CONVENTIONS

- `ProviderInfo.id` is the key. `name` is display text and can differ for local presets.
- Empty unconfigured custom provider groups stay hidden; custom providers are user-defined config entries, not a catalog list.
- Provider icons are lazy-loaded through `getProviderIcon`, `convertFileSrc`, an in-memory path cache, and a failure TTL in localStorage.
- Icon request concurrency is capped at 4 in `ProviderList.tsx`.
- Base URL-capable providers are decided by backend fields, not frontend provider ID lists.

## ANTI-PATTERNS

- Do not eagerly request all provider icons during initial render.
- Do not show delete auth controls unless `provider.can_delete_auth` is true.
- Do not duplicate the provider preset list in frontend code; backend owns preset metadata and provider classification.
- Do not create an empty custom group just because the section exists in the component API.
