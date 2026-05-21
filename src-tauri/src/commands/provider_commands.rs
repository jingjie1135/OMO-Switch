use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;

use crate::services::{provider_service, provider_store};

const PROVIDER_DOMAINS: &[(&str, &str)] = &[
    ("anthropic", "anthropic.com"),
    ("openai", "openai.com"),
    ("google", "google.com"),
    ("groq", "groq.com"),
    ("openrouter", "openrouter.ai"),
    ("mistral", "mistral.ai"),
    ("cohere", "cohere.com"),
    ("deepseek", "deepseek.com"),
    ("xai", "x.ai"),
    ("cerebras", "cerebras.ai"),
    ("perplexity", "perplexity.ai"),
    ("togetherai", "together.xyz"),
    ("deepinfra", "deepinfra.com"),
    ("azure", "azure.microsoft.com"),
    ("amazon-bedrock", "aws.amazon.com"),
    ("github-copilot", "github.com"),
    ("vercel", "vercel.com"),
    ("gitlab", "gitlab.com"),
    ("aicodewith", "aicodewith.com"),
    ("kimi-for-coding", "moonshot.cn"),
    ("zhipuai", "bigmodel.cn"),
    ("zhipuai-coding-plan", "bigmodel.cn"),
    ("moonshotai", "moonshot.cn"),
    ("moonshotai-cn", "moonshot.cn"),
    ("opencode", "opencode.ai"),
];

pub type ProviderInfo = provider_service::ProviderInfo;
pub type ProviderConfigSnapshot = provider_service::ProviderConfigSnapshot;
pub type ConnectionTestResult = provider_service::ConnectionTestResult;
#[cfg(test)]
pub(crate) type AuthEntry = provider_store::AuthEntry;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModelLimit {
    pub context: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input: Option<u64>,
    pub output: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CustomModelMetadata {
    pub limit: Option<ModelLimit>,
}

fn get_provider_icon_cache_path(provider_id: &str) -> Result<std::path::PathBuf, String> {
    provider_store::get_provider_icon_cache_path(provider_id)
}

fn validate_model_limit(limit: &ModelLimit) -> Result<(), String> {
    if limit.context == 0 {
        return Err("context 必须是大于 0 的整数".to_string());
    }
    if matches!(limit.input, Some(0)) {
        return Err("input 必须是大于 0 的整数".to_string());
    }
    if limit.output == 0 {
        return Err("output 必须是大于 0 的整数".to_string());
    }
    Ok(())
}

fn custom_model_config<'a>(
    config: &'a Value,
    provider_id: &str,
    model_id: &str,
) -> Result<&'a serde_json::Map<String, Value>, String> {
    let providers = config
        .get("provider")
        .and_then(|value| value.as_object())
        .ok_or("配置文件中不存在 provider 字段")?;
    let provider = providers
        .get(provider_id)
        .ok_or(format!("供应商 {} 不存在", provider_id))?;
    let models = provider
        .get("models")
        .and_then(|value| value.as_object())
        .ok_or(format!("供应商 {} 没有配置任何模型", provider_id))?;
    let model = models.get(model_id).ok_or(format!(
        "模型 {} 在供应商 {} 中不存在",
        model_id, provider_id
    ))?;

    model.as_object().ok_or(format!(
        "模型 {} 在供应商 {} 中的配置格式错误",
        model_id, provider_id
    ))
}

fn custom_model_config_mut<'a>(
    config: &'a mut Value,
    provider_id: &str,
    model_id: &str,
) -> Result<&'a mut serde_json::Map<String, Value>, String> {
    let providers = config
        .get_mut("provider")
        .and_then(|value| value.as_object_mut())
        .ok_or("配置文件中不存在 provider 字段")?;
    let provider = providers
        .get_mut(provider_id)
        .ok_or(format!("供应商 {} 不存在", provider_id))?;
    let models = provider
        .get_mut("models")
        .and_then(|value| value.as_object_mut())
        .ok_or(format!("供应商 {} 没有配置任何模型", provider_id))?;
    let model = models.get_mut(model_id).ok_or(format!(
        "模型 {} 在供应商 {} 中不存在",
        model_id, provider_id
    ))?;

    model.as_object_mut().ok_or(format!(
        "模型 {} 在供应商 {} 中的配置格式错误",
        model_id, provider_id
    ))
}

#[tauri::command]
pub fn get_provider_status() -> Result<Vec<ProviderInfo>, String> {
    provider_service::get_provider_status()
}

#[tauri::command]
pub fn get_provider_config(provider_id: String) -> Result<ProviderConfigSnapshot, String> {
    provider_service::get_provider_config(provider_id)
}

#[tauri::command]
pub fn test_provider_connection(
    npm: String,
    base_url: Option<String>,
    api_key: String,
) -> Result<ConnectionTestResult, String> {
    provider_service::test_provider_connection(npm, base_url, api_key)
}

#[tauri::command]
pub fn set_provider_api_key(
    provider_id: String,
    api_key: String,
    base_url: Option<String>,
    provider_type: Option<String>,
) -> Result<(), String> {
    provider_service::set_provider_api_key(provider_id, api_key, base_url, provider_type)
}

#[tauri::command]
pub fn delete_provider_auth(provider_id: String) -> Result<(), String> {
    provider_service::delete_provider_auth(provider_id)
}

#[tauri::command]
pub fn add_custom_provider(
    name: String,
    api_key: String,
    base_url: String,
) -> Result<ProviderInfo, String> {
    provider_service::add_custom_provider(name, api_key, base_url)
}

#[tauri::command]
pub fn add_custom_model(provider_id: String, model_id: String) -> Result<(), String> {
    let provider_id = provider_id.trim().to_string();
    if provider_id.is_empty() {
        return Err("供应商 ID 不能为空".to_string());
    }

    let model_id = model_id.trim().to_string();
    if model_id.is_empty() {
        return Err("模型 ID 不能为空".to_string());
    }

    let mut config = provider_store::read_opencode_config()?;
    let providers = config
        .get_mut("provider")
        .and_then(Value::as_object_mut)
        .ok_or("配置文件中不存在 provider 字段")?;
    let provider = providers
        .get_mut(&provider_id)
        .ok_or(format!("供应商 {} 不存在", provider_id))?;
    let provider_config = provider
        .as_object_mut()
        .ok_or(format!("供应商 {} 配置格式错误", provider_id))?;
    let models = provider_config
        .entry("models".to_string())
        .or_insert_with(|| json!({}))
        .as_object_mut()
        .ok_or("models 字段格式错误")?;

    models.entry(model_id).or_insert_with(|| json!({}));

    provider_store::write_opencode_config(&config)?;
    Ok(())
}

#[tauri::command]
pub fn remove_custom_model(provider_id: String, model_id: String) -> Result<(), String> {
    let mut config = provider_store::read_opencode_config()?;

    let provider = config
        .get("provider")
        .ok_or("配置文件中不存在 provider 字段")?;
    let provider_config = provider
        .get(&provider_id)
        .ok_or(format!("供应商 {} 不存在", provider_id))?;
    let models = provider_config
        .get("models")
        .ok_or(format!("供应商 {} 没有配置任何模型", provider_id))?;

    if models.get(&model_id).is_none() {
        return Err(format!(
            "模型 {} 在供应商 {} 中不存在",
            model_id, provider_id
        ));
    }

    config["provider"][&provider_id]["models"]
        .as_object_mut()
        .ok_or("models 字段格式错误")?
        .remove(&model_id);

    provider_store::write_opencode_config(&config)?;
    Ok(())
}

#[tauri::command]
pub fn get_custom_models() -> Result<HashMap<String, Vec<String>>, String> {
    Ok(provider_store::get_custom_models())
}

#[tauri::command]
pub fn get_custom_model_metadata(
    provider_id: String,
    model_id: String,
) -> Result<CustomModelMetadata, String> {
    let config = provider_store::read_opencode_config()?;
    let model = custom_model_config(&config, &provider_id, &model_id)?;

    let limit = match model.get("limit") {
        Some(value) => Some(
            serde_json::from_value::<ModelLimit>(value.clone())
                .map_err(|e| format!("解析模型 limit 失败: {}", e))?,
        ),
        None => None,
    };

    Ok(CustomModelMetadata { limit })
}

#[tauri::command]
pub fn update_custom_model_limit(
    provider_id: String,
    model_id: String,
    limit: ModelLimit,
) -> Result<(), String> {
    validate_model_limit(&limit)?;

    let mut config = provider_store::read_opencode_config()?;
    let model = custom_model_config_mut(&mut config, &provider_id, &model_id)?;
    let limit_value =
        serde_json::to_value(limit).map_err(|e| format!("序列化模型 limit 失败: {}", e))?;
    model.insert("limit".to_string(), limit_value);

    provider_store::write_opencode_config(&config)
}

#[tauri::command]
pub fn get_provider_icon(provider_id: String) -> Result<Option<String>, String> {
    let cache_path = get_provider_icon_cache_path(&provider_id)?;
    if cache_path.exists() {
        return Ok(Some(cache_path.to_string_lossy().to_string()));
    }

    let domain = PROVIDER_DOMAINS
        .iter()
        .find(|(id, _)| *id == provider_id)
        .map(|(_, domain)| *domain);

    let Some(domain) = domain else {
        return Ok(None);
    };

    let url = format!("https://logo.clearbit.com/{}?size=64", domain);
    let response = ureq::get(&url)
        .timeout(std::time::Duration::from_secs(5))
        .call();

    match response {
        Ok(resp) if resp.status() == 200 => {
            use std::io::Read;
            let mut bytes = Vec::new();
            resp.into_reader()
                .read_to_end(&mut bytes)
                .map_err(|e| format!("读取响应失败: {}", e))?;

            if let Some(parent) = cache_path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }

            std::fs::write(&cache_path, &bytes).map_err(|e| format!("写入缓存失败: {}", e))?;
            Ok(Some(cache_path.to_string_lossy().to_string()))
        }
        _ => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;
    use serial_test::serial;

    fn with_temp_home<T>(name: &str, test: impl FnOnce(&std::path::Path) -> T) -> T {
        let temp_dir = std::env::temp_dir().join(name);
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).expect("创建临时目录失败");

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

    fn write_opencode_fixture(temp_dir: &std::path::Path, content: &str) {
        let config_dir = temp_dir.join(".config").join("opencode");
        std::fs::create_dir_all(&config_dir).expect("创建配置目录失败");
        std::fs::write(config_dir.join("opencode.json"), content).expect("写入配置文件失败");
    }

    fn read_opencode_fixture(temp_dir: &std::path::Path) -> Value {
        let config_path = temp_dir
            .join(".config")
            .join("opencode")
            .join("opencode.json");
        let content = std::fs::read_to_string(config_path).expect("读取配置文件失败");
        serde_json::from_str(&content).expect("解析配置文件失败")
    }

    #[test]
    fn test_provider_info_serialization() {
        let provider = ProviderInfo {
            id: "test".to_string(),
            name: "Test Provider".to_string(),
            npm: Some("@test/provider".to_string()),
            website_url: Some("https://test.com".to_string()),
            is_configured: true,
            is_builtin: true,
            supports_base_url: true,
            supports_connection_test: true,
            can_delete_auth: true,
        };

        let json = serde_json::to_string(&provider).unwrap();
        assert!(json.contains("test"));
        assert!(json.contains("Test Provider"));
    }

    #[test]
    fn test_connection_test_result_serialization() {
        let result = ConnectionTestResult {
            success: true,
            message: "OK".to_string(),
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("success"));
        assert!(json.contains("OK"));
    }

    #[test]
    fn test_auth_entry_serialization() {
        let mut auth = HashMap::new();
        auth.insert(
            "test".to_string(),
            AuthEntry {
                auth_type: Some("api".to_string()),
                key: Some("sk-test".to_string()),
                extra: HashMap::new(),
            },
        );

        let json = serde_json::to_string(&auth).unwrap();
        assert!(json.contains("test"));
        assert!(json.contains("sk-test"));
    }

    #[test]
    fn test_auth_entry_deserialize_oauth_without_key() {
        let json = r#"{
            "openai": {
                "type": "oauth",
                "refresh": "rt_xxx",
                "access": "at_xxx"
            }
        }"#;

        let auth: HashMap<String, AuthEntry> = serde_json::from_str(json).unwrap();
        let openai = auth.get("openai").expect("openai should exist");

        assert_eq!(openai.auth_type.as_deref(), Some("oauth"));
        assert_eq!(openai.key, None);
        assert!(openai.extra.contains_key("refresh"));
        assert!(openai.extra.contains_key("access"));
    }

    #[test]
    #[serial]
    fn test_get_custom_model_metadata_returns_existing_limit() {
        with_temp_home("omo_test_get_custom_model_metadata_limit", |temp_dir| {
            write_opencode_fixture(
                temp_dir,
                r#"{
                  "provider": {
                    "test-provider": {
                      "models": {
                        "test-model": {
                          "limit": {
                            "context": 128000,
                            "input": 120000,
                            "output": 8192
                          }
                        }
                      }
                    }
                  }
                }"#,
            );

            let metadata =
                get_custom_model_metadata("test-provider".to_string(), "test-model".to_string())
                    .expect("读取模型 metadata 应该成功");

            let limit = metadata.limit.expect("应该返回已有 limit");
            assert_eq!(limit.context, 128000);
            assert_eq!(limit.input, Some(120000));
            assert_eq!(limit.output, 8192);
        });
    }

    #[test]
    #[serial]
    fn test_get_custom_model_metadata_returns_no_limit_for_empty_model() {
        with_temp_home("omo_test_get_custom_model_metadata_empty", |temp_dir| {
            write_opencode_fixture(
                temp_dir,
                r#"{
                  "provider": {
                    "test-provider": {
                      "models": {
                        "test-model": {}
                      }
                    }
                  }
                }"#,
            );

            let metadata =
                get_custom_model_metadata("test-provider".to_string(), "test-model".to_string())
                    .expect("读取空模型 metadata 应该成功");

            assert!(metadata.limit.is_none());
        });
    }

    #[test]
    #[serial]
    fn test_update_custom_model_limit_writes_limit_and_preserves_model_fields() {
        with_temp_home("omo_test_update_custom_model_limit_preserve", |temp_dir| {
            write_opencode_fixture(
                temp_dir,
                r#"{
                  "provider": {
                    "test-provider": {
                      "npm": "@ai-sdk/openai-compatible",
                      "models": {
                        "test-model": {
                          "name": "Preserved Name",
                          "capabilities": { "tools": true }
                        }
                      }
                    }
                  }
                }"#,
            );

            update_custom_model_limit(
                "test-provider".to_string(),
                "test-model".to_string(),
                ModelLimit {
                    context: 200000,
                    input: Some(180000),
                    output: 16000,
                },
            )
            .expect("更新模型 limit 应该成功");

            let config = read_opencode_fixture(temp_dir);
            let model = &config["provider"]["test-provider"]["models"]["test-model"];
            assert_eq!(model["limit"]["context"], 200000);
            assert_eq!(model["limit"]["input"], 180000);
            assert_eq!(model["limit"]["output"], 16000);
            assert_eq!(model["name"], "Preserved Name");
            assert_eq!(model["capabilities"]["tools"], true);
        });
    }

    #[test]
    #[serial]
    fn test_update_custom_model_limit_omits_empty_input() {
        with_temp_home("omo_test_update_custom_model_limit_no_input", |temp_dir| {
            write_opencode_fixture(
                temp_dir,
                r#"{
                  "provider": {
                    "test-provider": {
                      "models": {
                        "test-model": {}
                      }
                    }
                  }
                }"#,
            );

            update_custom_model_limit(
                "test-provider".to_string(),
                "test-model".to_string(),
                ModelLimit {
                    context: 128000,
                    input: None,
                    output: 8192,
                },
            )
            .expect("更新无 input 的模型 limit 应该成功");

            let config = read_opencode_fixture(temp_dir);
            let limit = &config["provider"]["test-provider"]["models"]["test-model"]["limit"];
            assert_eq!(limit["context"], 128000);
            assert!(limit.get("input").is_none());
            assert_eq!(limit["output"], 8192);
        });
    }

    #[test]
    #[serial]
    fn test_update_custom_model_limit_missing_model_returns_error() {
        with_temp_home("omo_test_update_custom_model_limit_missing", |temp_dir| {
            write_opencode_fixture(
                temp_dir,
                r#"{
                  "provider": {
                    "test-provider": {
                      "models": {
                        "existing-model": {}
                      }
                    }
                  }
                }"#,
            );

            let error = update_custom_model_limit(
                "test-provider".to_string(),
                "missing-model".to_string(),
                ModelLimit {
                    context: 128000,
                    input: Some(120000),
                    output: 8192,
                },
            )
            .expect_err("缺失模型应该返回错误");

            assert!(error.contains("missing-model"));
            assert!(error.contains("不存在"));
        });
    }

    #[test]
    fn test_test_provider_connection_rejects_invalid_base_url() {
        let result = test_provider_connection(
            "@ai-sdk/openai".to_string(),
            Some("ftp://invalid.example.com".to_string()),
            "sk-test".to_string(),
        )
        .unwrap();

        assert!(!result.success);
        assert!(result.message.contains("Base URL"));
    }

    #[test]
    fn test_test_provider_connection_accepts_valid_payload() {
        let result = test_provider_connection(
            "@ai-sdk/openai".to_string(),
            Some("https://api.openai.com/v1".to_string()),
            "sk-test".to_string(),
        )
        .unwrap();

        assert!(result.success);
    }

    #[test]
    #[serial]
    fn test_get_provider_status_graceful_when_auth_invalid() {
        let temp_dir = std::env::temp_dir().join("omo_test_provider_status_auth_invalid");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).expect("创建临时目录失败");

        let original_home = std::env::var("HOME").ok();
        let original_userprofile = std::env::var("USERPROFILE").ok();
        unsafe {
            std::env::set_var("HOME", &temp_dir);
            std::env::set_var("USERPROFILE", &temp_dir);
        }

        let cache_dir = temp_dir.join(".cache").join("oh-my-opencode");
        std::fs::create_dir_all(&cache_dir).expect("创建缓存目录失败");
        std::fs::write(
            cache_dir.join("provider-models.json"),
            r#"{"models":{"openai":["gpt-5"]}}"#,
        )
        .expect("写入 provider-models.json 失败");
        std::fs::write(
            cache_dir.join("connected-providers.json"),
            r#"{"connected":[],"updatedAt":"2026-02-24T00:00:00.000Z"}"#,
        )
        .expect("写入 connected-providers.json 失败");

        let auth_dir = temp_dir.join(".local").join("share").join("opencode");
        std::fs::create_dir_all(&auth_dir).expect("创建 auth 目录失败");
        std::fs::write(auth_dir.join("auth.json"), "{invalid json").expect("写入 auth.json 失败");

        let result = get_provider_status();

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

        assert!(
            result.is_ok(),
            "auth.json 异常时应降级，不应阻断 provider 状态"
        );
        let providers = result.unwrap();
        let openai = providers
            .iter()
            .find(|provider| provider.id == "openai")
            .expect("openai should remain visible");
        assert!(!openai.is_configured);

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    #[serial]
    fn test_add_custom_model() {
        let temp_dir = std::env::temp_dir().join("omo_test_add_model");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).expect("创建临时目录失败");

        let original_home = std::env::var("HOME").ok();
        let original_userprofile = std::env::var("USERPROFILE").ok();
        unsafe {
            std::env::set_var("HOME", &temp_dir);
            std::env::set_var("USERPROFILE", &temp_dir);
        }

        write_opencode_fixture(
            &temp_dir,
            r#"{"provider":{"test-provider":{"npm":"@ai-sdk/openai-compatible"}}}"#,
        );

        let result = add_custom_model("test-provider".to_string(), "test-model-1".to_string());

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

        assert!(result.is_ok(), "添加模型应该成功: {:?}", result.err());

        let config_path = temp_dir
            .join(".config")
            .join("opencode")
            .join("opencode.json");
        assert!(config_path.exists(), "配置文件应该被创建");

        let content = std::fs::read_to_string(&config_path).expect("读取配置文件失败");
        let config: Value = serde_json::from_str(&content).expect("解析配置文件失败");
        assert!(config["provider"]["test-provider"]["models"]["test-model-1"].is_object());

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    #[serial]
    fn test_add_custom_model_duplicate() {
        let temp_dir = std::env::temp_dir().join("omo_test_add_model_dup");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).expect("创建临时目录失败");

        let original_home = std::env::var("HOME").ok();
        let original_userprofile = std::env::var("USERPROFILE").ok();
        unsafe {
            std::env::set_var("HOME", &temp_dir);
            std::env::set_var("USERPROFILE", &temp_dir);
        }

        write_opencode_fixture(
            &temp_dir,
            r#"{"provider":{"test-provider":{"npm":"@ai-sdk/openai-compatible"}}}"#,
        );

        let result1 = add_custom_model("test-provider".to_string(), "test-model-2".to_string());
        assert!(result1.is_ok());
        let result2 = add_custom_model("test-provider".to_string(), "test-model-2".to_string());
        assert!(result2.is_ok());

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

        let config_path = temp_dir
            .join(".config")
            .join("opencode")
            .join("opencode.json");
        let content = std::fs::read_to_string(&config_path).expect("读取配置文件失败");
        let config: Value = serde_json::from_str(&content).expect("解析配置文件失败");

        let models = config["provider"]["test-provider"]["models"]
            .as_object()
            .unwrap();
        assert_eq!(models.len(), 1);
        assert!(models.contains_key("test-model-2"));

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    #[serial]
    fn test_add_custom_model_trims_model_id() {
        with_temp_home("omo_test_add_model_trim", |temp_dir| {
            write_opencode_fixture(
                temp_dir,
                r#"{"provider":{"test-provider":{"npm":"@ai-sdk/openai-compatible"}}}"#,
            );

            let result =
                add_custom_model("test-provider".to_string(), "  spaced-model  ".to_string());

            assert!(result.is_ok(), "添加模型应该成功: {:?}", result.err());

            let config = read_opencode_fixture(temp_dir);
            let models = config["provider"]["test-provider"]["models"]
                .as_object()
                .unwrap();
            assert!(models.contains_key("spaced-model"));
            assert!(!models.contains_key("  spaced-model  "));
        });
    }

    #[test]
    #[serial]
    fn test_add_custom_model_rejects_empty_model_id() {
        with_temp_home("omo_test_add_model_empty", |_| {
            let error = add_custom_model("test-provider".to_string(), "   ".to_string())
                .expect_err("空模型 ID 应该失败");

            assert!(error.contains("模型 ID"));
        });
    }

    #[test]
    #[serial]
    fn test_add_custom_model_rejects_missing_provider() {
        with_temp_home("omo_test_add_model_missing_provider", |temp_dir| {
            write_opencode_fixture(
                temp_dir,
                r#"{"provider":{"other-provider":{"npm":"@ai-sdk/openai-compatible"}}}"#,
            );

            let error = add_custom_model("missing-provider".to_string(), "test-model".to_string())
                .expect_err("缺失供应商应该失败");

            assert!(error.contains("missing-provider"));
            assert!(error.contains("不存在"));
        });
    }

    #[test]
    #[serial]
    fn test_remove_custom_model() {
        let temp_dir = std::env::temp_dir().join("omo_test_remove_model");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).expect("创建临时目录失败");

        let original_home = std::env::var("HOME").ok();
        let original_userprofile = std::env::var("USERPROFILE").ok();
        unsafe {
            std::env::set_var("HOME", &temp_dir);
            std::env::set_var("USERPROFILE", &temp_dir);
        }

        write_opencode_fixture(
            &temp_dir,
            r#"{"provider":{"test-provider":{"npm":"@ai-sdk/openai-compatible"}}}"#,
        );

        let add_result = add_custom_model("test-provider".to_string(), "test-model-3".to_string());
        assert!(add_result.is_ok());
        let remove_result =
            remove_custom_model("test-provider".to_string(), "test-model-3".to_string());
        assert!(
            remove_result.is_ok(),
            "删除模型应该成功: {:?}",
            remove_result.err()
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

        let config_path = temp_dir
            .join(".config")
            .join("opencode")
            .join("opencode.json");
        let content = std::fs::read_to_string(&config_path).expect("读取配置文件失败");
        let config: Value = serde_json::from_str(&content).expect("解析配置文件失败");

        let models = config["provider"]["test-provider"]["models"]
            .as_object()
            .unwrap();
        assert!(!models.contains_key("test-model-3"));

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    #[serial]
    fn test_remove_custom_model_not_found() {
        let temp_dir = std::env::temp_dir().join("omo_test_remove_not_found");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).expect("创建临时目录失败");

        let original_home = std::env::var("HOME").ok();
        let original_userprofile = std::env::var("USERPROFILE").ok();
        unsafe {
            std::env::set_var("HOME", &temp_dir);
            std::env::set_var("USERPROFILE", &temp_dir);
        }

        write_opencode_fixture(
            &temp_dir,
            r#"{"provider":{"test-provider":{"npm":"@ai-sdk/openai-compatible"}}}"#,
        );

        let _ = add_custom_model("test-provider".to_string(), "existing-model".to_string());
        let result =
            remove_custom_model("test-provider".to_string(), "nonexistent-model".to_string());

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

        assert!(result.is_err());
        let error_msg = result.unwrap_err();
        assert!(error_msg.contains("不存在") || error_msg.contains("nonexistent"));

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
