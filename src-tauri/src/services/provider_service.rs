use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};

use crate::services::provider_store;
use crate::services::provider_store::AuthEntry;

const BASE_URL_COMPATIBLE_PROVIDERS: &[&str] = &[
    "openai",
    "deepseek",
    "groq",
    "openrouter",
    "xai",
    "moonshotai",
    "moonshotai-cn",
    "kimi-for-coding",
    "zhipuai",
    "zhipuai-coding-plan",
    "minimax",
    "minimax-cn",
    "minimax-coding-plan",
    "minimax-cn-coding-plan",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderInfo {
    pub id: String,
    pub name: String,
    pub npm: Option<String>,
    pub website_url: Option<String>,
    pub is_configured: bool,
    pub is_builtin: bool,
    pub supports_base_url: bool,
    pub supports_connection_test: bool,
    pub can_delete_auth: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfigSnapshot {
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub provider_type: Option<String>,
    pub default_provider_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionTestResult {
    pub success: bool,
    pub message: String,
}

pub fn provider_default_npm(provider_id: &str) -> &'static str {
    match provider_id {
        "openai" => "@ai-sdk/openai",
        "github-copilot" => "@ai-sdk/github-copilot",
        "zhipuai"
        | "zhipuai-coding-plan"
        | "moonshotai"
        | "moonshotai-cn"
        | "kimi-for-coding"
        | "minimax"
        | "minimax-cn"
        | "minimax-coding-plan"
        | "minimax-cn-coding-plan" => "@ai-sdk/openai-compatible",
        "deepseek" => "@ai-sdk/anthropic",
        "xai" => "@ai-sdk/openai",
        "groq" => "@ai-sdk/groq",
        "openrouter" => "@openrouter/ai-sdk-provider",
        _ => "@ai-sdk/openai",
    }
}

pub fn provider_supports_base_url(provider_id: &str) -> bool {
    provider_id != "opencode"
}

pub fn provider_supports_connection_test(provider_id: &str) -> bool {
    BASE_URL_COMPATIBLE_PROVIDERS.contains(&provider_id)
}

pub fn is_valid_base_url(url: &str) -> bool {
    let trimmed = url.trim();
    !trimmed.is_empty()
        && (trimmed.starts_with("https://") || trimmed.starts_with("http://"))
        && trimmed.contains("://")
}

fn get_provider_base_url(provider_id: &str, config: &Value) -> Option<String> {
    config
        .get("provider")
        .and_then(|providers| providers.get(provider_id))
        .and_then(|provider| provider.get("options"))
        .and_then(|options| options.get("baseURL").or_else(|| options.get("baseUrl")))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
}

fn get_provider_npm(provider_id: &str, config: &Value) -> Option<String> {
    config
        .get("provider")
        .and_then(|providers| providers.get(provider_id))
        .and_then(|provider| provider.get("npm"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
}

fn configured_provider_ids(config: &Value) -> HashSet<String> {
    config
        .get("provider")
        .and_then(Value::as_object)
        .map(|providers| providers.keys().cloned().collect())
        .unwrap_or_default()
}

fn provider_has_custom_shape(config: &Value, provider_id: &str) -> bool {
    let Some(provider) = config
        .get("provider")
        .and_then(Value::as_object)
        .and_then(|providers| providers.get(provider_id))
    else {
        return false;
    };

    let has_openai_compatible_npm = provider
        .get("npm")
        .and_then(Value::as_str)
        .map(str::trim)
        .is_some_and(|npm| npm == "@ai-sdk/openai-compatible");
    let has_base_url = provider
        .get("options")
        .and_then(Value::as_object)
        .and_then(|options| options.get("baseURL").or_else(|| options.get("baseUrl")))
        .and_then(Value::as_str)
        .map(str::trim)
        .is_some_and(|base_url| !base_url.is_empty());
    let has_models = provider.get("models").and_then(Value::as_object).is_some();

    has_openai_compatible_npm || has_base_url || has_models
}

fn build_builtin_provider_ids(
    provider_model_catalog_ids: &HashSet<String>,
    builtin_presets: &HashMap<String, provider_store::ProviderPresetEntry>,
) -> HashSet<String> {
    let mut builtin_provider_ids: HashSet<String> = builtin_presets.keys().cloned().collect();
    builtin_provider_ids.extend(provider_model_catalog_ids.iter().cloned());
    builtin_provider_ids
}

fn provider_is_builtin(
    provider_id: &str,
    config: &Value,
    config_provider_ids: &HashSet<String>,
    builtin_provider_ids: &HashSet<String>,
) -> bool {
    if builtin_provider_ids.contains(provider_id) {
        return true;
    }

    !(config_provider_ids.contains(provider_id) && provider_has_custom_shape(config, provider_id))
}

pub fn get_provider_status() -> Result<Vec<ProviderInfo>, String> {
    let provider_models = provider_store::read_provider_models()?;
    let provider_model_catalog_ids = match provider_store::read_provider_model_catalog_ids() {
        Ok(ids) => ids,
        Err(err) => {
            eprintln!(
                "警告：读取 provider-models.json provider catalog 失败，降级使用本地 metadata: {}",
                err
            );
            HashSet::new()
        }
    };
    let connected = provider_store::read_connected_providers()?;
    let auth_data = match provider_store::read_auth_file() {
        Ok(data) => data,
        Err(err) => {
            eprintln!("警告：读取 auth.json 失败，降级为空认证数据: {}", err);
            HashMap::new()
        }
    };
    let config = match provider_store::read_opencode_config() {
        Ok(config) => config,
        Err(err) => {
            eprintln!(
                "警告：读取 opencode provider 失败，降级为空配置数据: {}",
                err
            );
            json!({})
        }
    };
    let config_provider_ids = configured_provider_ids(&config);
    let builtin_presets = provider_store::load_builtin_provider_presets();
    let builtin_provider_ids =
        build_builtin_provider_ids(&provider_model_catalog_ids, &builtin_presets);

    let mut provider_ids: HashSet<String> = builtin_provider_ids.clone();
    provider_ids.extend(provider_models.keys().cloned());
    provider_ids.extend(connected.iter().cloned());
    provider_ids.extend(auth_data.keys().cloned());
    provider_ids.extend(config_provider_ids.iter().cloned());

    let mut providers = Vec::new();
    for provider_id in provider_ids {
        let preset = builtin_presets.get(&provider_id);
        let has_auth = auth_data.contains_key(&provider_id);
        let is_configured = connected.contains(&provider_id)
            || has_auth
            || config_provider_ids.contains(&provider_id);
        providers.push(ProviderInfo {
            id: provider_id.clone(),
            name: preset
                .map(|entry| entry.name.clone())
                .unwrap_or_else(|| provider_id.clone()),
            npm: preset.and_then(|entry| entry.npm.clone()),
            website_url: preset.and_then(|entry| entry.website_url.clone()),
            is_configured,
            is_builtin: provider_is_builtin(
                &provider_id,
                &config,
                &config_provider_ids,
                &builtin_provider_ids,
            ),
            supports_base_url: provider_supports_base_url(&provider_id),
            supports_connection_test: provider_supports_connection_test(&provider_id),
            can_delete_auth: has_auth,
        });
    }

    providers.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(providers)
}

pub fn get_provider_config(provider_id: String) -> Result<ProviderConfigSnapshot, String> {
    let auth_data = match provider_store::read_auth_file() {
        Ok(data) => data,
        Err(err) => {
            eprintln!("警告：读取 auth.json 失败，降级为空认证数据: {}", err);
            HashMap::new()
        }
    };
    let config = provider_store::read_opencode_config()?;

    let api_key = auth_data
        .get(&provider_id)
        .and_then(|entry| entry.key.clone())
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());

    Ok(ProviderConfigSnapshot {
        api_key,
        base_url: get_provider_base_url(&provider_id, &config),
        provider_type: get_provider_npm(&provider_id, &config),
        default_provider_type: provider_default_npm(&provider_id).to_string(),
    })
}

pub fn test_provider_connection(
    npm: String,
    base_url: Option<String>,
    api_key: String,
) -> Result<ConnectionTestResult, String> {
    if api_key.trim().is_empty() {
        return Ok(ConnectionTestResult {
            success: false,
            message: "API Key 不能为空".to_string(),
        });
    }

    if let Some(url) = base_url
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        if !is_valid_base_url(url) {
            return Ok(ConnectionTestResult {
                success: false,
                message: "Base URL 必须以 http:// 或 https:// 开头".to_string(),
            });
        }
    }

    if !npm.trim().is_empty() && !npm.trim().starts_with('@') {
        return Ok(ConnectionTestResult {
            success: false,
            message: "Provider npm 标识格式无效".to_string(),
        });
    }

    Ok(ConnectionTestResult {
        success: true,
        message: "配置校验通过".to_string(),
    })
}

pub fn set_provider_api_key(
    provider_id: String,
    api_key: String,
    base_url: Option<String>,
    provider_type: Option<String>,
) -> Result<(), String> {
    if api_key.trim().is_empty() {
        return Err("API Key 不能为空".to_string());
    }

    let provider_id_for_config = provider_id.clone();
    let auth_existed = provider_store::get_auth_file_path()?.exists();
    let original_auth = provider_store::read_auth_file()?;
    let mut auth_data = original_auth.clone();

    auth_data.insert(
        provider_id,
        AuthEntry {
            auth_type: Some("api".to_string()),
            key: Some(api_key),
            extra: HashMap::new(),
        },
    );

    provider_store::write_auth_file(&auth_data)?;

    if provider_supports_base_url(&provider_id_for_config) {
        let original_config = provider_store::read_opencode_config()?;
        let mut config = original_config.clone();
        if config.get("provider").is_none() {
            config["provider"] = json!({});
        }
        if config["provider"].get(&provider_id_for_config).is_none() {
            config["provider"][&provider_id_for_config] = json!({
                "npm": provider_default_npm(&provider_id_for_config)
            });
        }

        let selected_provider_type = provider_type
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| provider_default_npm(&provider_id_for_config));
        config["provider"][&provider_id_for_config]["npm"] = json!(selected_provider_type);

        let trimmed = base_url
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty());
        if let Some(url) = trimmed {
            if !is_valid_base_url(url) {
                provider_store::restore_auth_state(auth_existed, &original_auth)?;
                return Err("Base URL 必须以 http:// 或 https:// 开头".to_string());
            }
            if config["provider"][&provider_id_for_config]
                .get("options")
                .is_none()
            {
                config["provider"][&provider_id_for_config]["options"] = json!({});
            }
            config["provider"][&provider_id_for_config]["options"]["baseURL"] = json!(url);
        } else if let Some(options) =
            config["provider"][&provider_id_for_config]["options"].as_object_mut()
        {
            options.remove("baseURL");
            if options.is_empty() {
                config["provider"][&provider_id_for_config]
                    .as_object_mut()
                    .and_then(|provider| provider.remove("options"));
            }
        }

        if let Err(err) = provider_store::write_opencode_config(&config) {
            provider_store::restore_auth_state(auth_existed, &original_auth)?;
            return Err(err);
        }
    }

    Ok(())
}

pub fn delete_provider_auth(provider_id: String) -> Result<(), String> {
    let mut auth_data = provider_store::read_auth_file()?;
    if auth_data.remove(&provider_id).is_none() {
        return Ok(());
    }
    provider_store::write_auth_file(&auth_data)
}

pub fn add_custom_provider(
    name: String,
    api_key: String,
    base_url: String,
) -> Result<ProviderInfo, String> {
    if name.trim().is_empty() {
        return Err("Provider 名称不能为空".to_string());
    }
    if api_key.trim().is_empty() {
        return Err("API Key 不能为空".to_string());
    }
    if !is_valid_base_url(&base_url) {
        return Err("Base URL 必须以 http:// 或 https:// 开头".to_string());
    }

    let provider_key = name.to_lowercase().replace(' ', "-").replace('_', "-");
    let auth_existed = provider_store::get_auth_file_path()?.exists();
    let config_existed = provider_store::get_opencode_config_path()?.exists();

    let original_config = provider_store::read_opencode_config()?;
    let mut config = original_config.clone();
    if config.get("provider").is_none() {
        config["provider"] = json!({});
    }

    config["provider"][&provider_key] = json!({
        "npm": "@ai-sdk/openai-compatible",
        "options": { "baseURL": base_url },
        "models": {}
    });

    provider_store::write_opencode_config(&config)?;

    let original_auth = provider_store::read_auth_file()?;
    let mut auth_data = original_auth.clone();
    auth_data.insert(
        provider_key.clone(),
        AuthEntry {
            auth_type: Some("api".to_string()),
            key: Some(api_key),
            extra: HashMap::new(),
        },
    );

    if let Err(err) = provider_store::write_auth_file(&auth_data) {
        provider_store::restore_opencode_config_state(config_existed, &original_config)?;
        provider_store::restore_auth_state(auth_existed, &original_auth)?;
        return Err(err);
    }

    Ok(ProviderInfo {
        id: provider_key,
        name,
        npm: Some("@ai-sdk/openai-compatible".to_string()),
        website_url: Some(base_url),
        is_configured: true,
        is_builtin: false,
        supports_base_url: true,
        supports_connection_test: true,
        can_delete_auth: true,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::path::Path;

    fn with_temp_home<T>(name: &str, test: impl FnOnce(&Path) -> T) -> T {
        let temp_dir = std::env::temp_dir().join(name);
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).unwrap();

        let original_home = std::env::var("HOME").ok();
        let original_userprofile = std::env::var("USERPROFILE").ok();
        unsafe {
            std::env::set_var("HOME", &temp_dir);
            std::env::set_var("USERPROFILE", &temp_dir);
        }

        let result = test(&temp_dir);

        unsafe {
            if let Some(home) = original_home {
                std::env::set_var("HOME", home);
            } else {
                std::env::remove_var("HOME");
            }
            if let Some(userprofile) = original_userprofile {
                std::env::set_var("USERPROFILE", userprofile);
            } else {
                std::env::remove_var("USERPROFILE");
            }
        }

        let _ = std::fs::remove_dir_all(&temp_dir);
        result
    }

    fn write_opencode_config(temp_dir: &Path, content: &str) {
        let config_dir = temp_dir.join(".config").join("opencode");
        std::fs::create_dir_all(&config_dir).unwrap();
        std::fs::write(config_dir.join("opencode.json"), content).unwrap();
    }

    fn write_provider_models_cache(temp_dir: &Path, content: &str) {
        let cache_dir = temp_dir.join(".cache").join("oh-my-opencode");
        std::fs::create_dir_all(&cache_dir).unwrap();
        std::fs::write(cache_dir.join("provider-models.json"), content).unwrap();
    }

    fn provider_by_id<'a>(providers: &'a [ProviderInfo], provider_id: &str) -> &'a ProviderInfo {
        providers
            .iter()
            .find(|provider| provider.id == provider_id)
            .unwrap_or_else(|| panic!("provider {provider_id} should exist"))
    }

    #[test]
    fn test_test_provider_connection_uses_validation_wording() {
        let result = test_provider_connection(
            "@ai-sdk/openai".to_string(),
            Some("https://api.openai.com/v1".to_string()),
            "sk-test".to_string(),
        )
        .unwrap();

        assert!(result.success);
        assert_eq!(result.message, "配置校验通过");
    }

    #[test]
    #[serial]
    fn test_get_provider_status_treats_provider_model_cache_provider_as_builtin() {
        with_temp_home("omo-provider-service-cache-builtin-test", |temp_dir| {
            write_provider_models_cache(
                temp_dir,
                r#"{
                  "models": {
                    "amazon-bedrock": [{
                      "id": "anthropic.claude-3-5-sonnet-20240620-v1:0",
                      "providerID": "amazon-bedrock",
                      "api": {
                        "id": "anthropic.claude-3-5-sonnet-20240620-v1:0",
                        "url": "",
                        "npm": "@ai-sdk/amazon-bedrock"
                      },
                      "name": "Claude 3.5 Sonnet",
                      "family": "claude-sonnet",
                      "release_date": "2024-06-20"
                    }]
                  }
                }"#,
            );

            let providers = get_provider_status().unwrap();
            let amazon_bedrock = provider_by_id(&providers, "amazon-bedrock");

            assert!(
                amazon_bedrock.is_builtin,
                "provider-model cache IDs come from OpenCode's catalog and must not be custom"
            );
        });
    }

    #[test]
    #[serial]
    fn test_get_provider_status_does_not_treat_verified_provider_model_cache_as_builtin() {
        with_temp_home("omo-provider-service-verified-custom-test", |temp_dir| {
            write_opencode_config(
                temp_dir,
                r#"{
                  "provider": {
                    "mirror0425": {
                      "npm": "@ai-sdk/openai-compatible",
                      "options": { "baseURL": "https://mirror.example.com/v1" },
                      "models": { "gpt-5.5": {} }
                    }
                  }
                }"#,
            );
            let cache_dir = temp_dir.join(".cache").join("oh-my-opencode");
            std::fs::create_dir_all(&cache_dir).unwrap();
            std::fs::write(
                cache_dir.join("verified-provider-models.json"),
                r#"{
                  "models": {
                    "mirror0425": ["gpt-5.5"]
                  }
                }"#,
            )
            .unwrap();

            let providers = get_provider_status().unwrap();
            let mirror = provider_by_id(&providers, "mirror0425");

            assert!(
                !mirror.is_builtin,
                "verified current-model output can include custom config providers and must not make them built-in"
            );
        });
    }

    #[test]
    #[serial]
    fn test_get_provider_status_marks_direct_non_catalog_config_provider_as_custom() {
        with_temp_home("omo-provider-service-direct-custom-test", |temp_dir| {
            write_opencode_config(
                temp_dir,
                r#"{
                  "provider": {
                    "mirror0425": {
                      "npm": "@ai-sdk/openai-compatible",
                      "options": { "baseURL": "https://mirror.example.com/v1" },
                      "models": { "gpt-5.5": {} }
                    }
                  }
                }"#,
            );

            let providers = get_provider_status().unwrap();
            let mirror = provider_by_id(&providers, "mirror0425");

            assert!(
                !mirror.is_builtin,
                "non-catalog providers explicitly defined in opencode.json should be custom"
            );
            assert!(
                mirror.is_configured,
                "providers explicitly defined in opencode.json should be shown as configured"
            );
        });
    }

    #[test]
    #[serial]
    fn test_get_provider_status_keeps_openai_compatible_alias_provider_custom() {
        with_temp_home("omo-provider-service-alias-custom-test", |temp_dir| {
            write_provider_models_cache(
                temp_dir,
                r#"{
                  "models": {
                    "ngle": [{
                      "id": "claude-opus-4-6-thinking",
                      "providerID": "ngle",
                      "api": {
                        "id": "claude-opus-4-6-thinking",
                        "url": "",
                        "npm": "@ai-sdk/openai-compatible"
                      },
                      "name": "claude-opus-4-6",
                      "family": "",
                      "release_date": ""
                    }]
                  }
                }"#,
            );
            write_opencode_config(
                temp_dir,
                r#"{
                  "provider": {
                    "ngle": {
                      "npm": "@ai-sdk/openai-compatible",
                      "options": { "baseURL": "https://mirror.example.com/v1" },
                      "models": {
                        "claude-opus-4-6-thinking": {
                          "name": "claude-opus-4-6"
                        }
                      }
                    }
                  }
                }"#,
            );

            let providers = get_provider_status().unwrap();
            let ngle = provider_by_id(&providers, "ngle");

            assert!(
                !ngle.is_builtin,
                "configured openai-compatible alias providers should remain custom even when cached model names differ from ids"
            );
        });
    }

    #[test]
    #[serial]
    fn test_get_provider_status_catalog_membership_wins_over_config_shape() {
        with_temp_home("omo-provider-service-catalog-wins-test", |temp_dir| {
            write_provider_models_cache(
                temp_dir,
                r#"{
                  "models": {
                    "amazon-bedrock": [{
                      "id": "anthropic.claude-3-5-sonnet-20240620-v1:0",
                      "providerID": "amazon-bedrock",
                      "api": {
                        "id": "anthropic.claude-3-5-sonnet-20240620-v1:0",
                        "url": "",
                        "npm": "@ai-sdk/amazon-bedrock"
                      },
                      "name": "Claude 3.5 Sonnet",
                      "family": "claude-sonnet",
                      "release_date": "2024-06-20"
                    }]
                  }
                }"#,
            );
            write_opencode_config(
                temp_dir,
                r#"{
                  "provider": {
                    "amazon-bedrock": {
                      "npm": "@ai-sdk/openai-compatible",
                      "options": { "baseURL": "https://bedrock-runtime.example.com" },
                      "models": { "custom-bedrock-alias": {} }
                    }
                  }
                }"#,
            );

            let providers = get_provider_status().unwrap();
            let amazon_bedrock = provider_by_id(&providers, "amazon-bedrock");

            assert!(
                amazon_bedrock.is_builtin,
                "built-in catalog membership must win over config overrides"
            );
        });
    }

    #[test]
    #[serial]
    fn test_get_provider_status_treats_auth_only_provider_as_builtin_known() {
        with_temp_home("omo-provider-service-auth-known-test", |temp_dir| {
            let auth_dir = temp_dir.join(".local").join("share").join("opencode");
            std::fs::create_dir_all(&auth_dir).unwrap();
            std::fs::write(
                auth_dir.join("auth.json"),
                r#"{
                  "oauth-only-provider": { "type": "oauth", "refresh": "rt", "access": "at" }
                }"#,
            )
            .unwrap();

            let providers = get_provider_status().unwrap();
            let provider = provider_by_id(&providers, "oauth-only-provider");

            assert!(
                provider.is_builtin,
                "providers discovered only from auth/cache data should be known built-ins, not custom"
            );
        });
    }

    #[test]
    #[serial]
    fn test_add_custom_provider_writes_models_object_without_private_marker() {
        with_temp_home(
            "omo-provider-service-add-custom-provider-shape-test",
            |temp_dir| {
                let provider = add_custom_provider(
                    "Mirror 0425".to_string(),
                    "sk-test".to_string(),
                    "https://mirror.example.com/v1".to_string(),
                )
                .unwrap();

                assert_eq!(provider.id, "mirror-0425");
                assert!(!provider.is_builtin);

                let config_path = temp_dir
                    .join(".config")
                    .join("opencode")
                    .join("opencode.json");
                let content = std::fs::read_to_string(config_path).unwrap();
                let config: Value = serde_json::from_str(&content).unwrap();
                let provider_config = &config["provider"]["mirror-0425"];

                assert_eq!(provider_config["npm"], "@ai-sdk/openai-compatible");
                assert_eq!(
                    provider_config["options"]["baseURL"],
                    "https://mirror.example.com/v1"
                );
                assert!(provider_config["models"].as_object().unwrap().is_empty());
                assert!(provider_config.get("omo_custom").is_none());
            },
        );
    }

    #[test]
    #[serial]
    fn test_get_provider_config_reads_legacy_base_url_key() {
        let temp_dir = std::env::temp_dir().join("omo-provider-service-legacy-baseurl-test");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).unwrap();

        let original_home = std::env::var("HOME").ok();
        let original_userprofile = std::env::var("USERPROFILE").ok();
        unsafe {
            std::env::set_var("HOME", &temp_dir);
            std::env::set_var("USERPROFILE", &temp_dir);
        }

        let config_dir = temp_dir.join(".config").join("opencode");
        std::fs::create_dir_all(&config_dir).unwrap();
        std::fs::write(
            config_dir.join("opencode.json"),
            r#"{
              "provider": {
                "openai": {
                  "npm": "@ai-sdk/openai",
                  "options": { "baseUrl": "https://legacy.example.com/v1" }
                }
              }
            }"#,
        )
        .unwrap();

        let auth_dir = temp_dir.join(".local").join("share").join("opencode");
        std::fs::create_dir_all(&auth_dir).unwrap();
        std::fs::write(
            auth_dir.join("auth.json"),
            r#"{"openai":{"type":"api","key":"sk-legacy"}}"#,
        )
        .unwrap();

        let snapshot = get_provider_config("openai".to_string()).unwrap();

        assert_eq!(snapshot.api_key.as_deref(), Some("sk-legacy"));
        assert_eq!(
            snapshot.base_url.as_deref(),
            Some("https://legacy.example.com/v1")
        );

        unsafe {
            if let Some(home) = original_home {
                std::env::set_var("HOME", home);
            } else {
                std::env::remove_var("HOME");
            }
            if let Some(userprofile) = original_userprofile {
                std::env::set_var("USERPROFILE", userprofile);
            } else {
                std::env::remove_var("USERPROFILE");
            }
        }

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
