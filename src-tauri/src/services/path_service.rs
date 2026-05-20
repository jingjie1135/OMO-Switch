use std::path::PathBuf;

pub const OMO_CACHE_DIR_NAME: &str = "oh-my-opencode";
pub const OMO_SWITCH_DIR_NAME: &str = "OMO-Switch";
pub const OPENCODE_DIR_NAME: &str = "opencode";

pub fn user_home_dir() -> Result<PathBuf, String> {
    #[cfg(windows)]
    {
        non_empty_env_path("USERPROFILE")
            .or_else(|| non_empty_env_path("HOME"))
            .or_else(dirs::home_dir)
            .ok_or_else(|| "无法获取用户主目录".to_string())
    }

    #[cfg(not(windows))]
    {
        non_empty_env_path("HOME")
            .or_else(dirs::home_dir)
            .or_else(|| non_empty_env_path("USERPROFILE"))
            .ok_or_else(|| "无法获取用户主目录".to_string())
    }
}

pub fn xdg_config_home() -> Result<PathBuf, String> {
    if let Some(path) = non_empty_env_path("XDG_CONFIG_HOME") {
        return Ok(path);
    }
    Ok(user_home_dir()?.join(".config"))
}

pub fn xdg_cache_home() -> Result<PathBuf, String> {
    if let Some(path) = non_empty_env_path("XDG_CACHE_HOME") {
        return Ok(path);
    }
    Ok(user_home_dir()?.join(".cache"))
}

pub fn xdg_data_home() -> Result<PathBuf, String> {
    if let Some(path) = non_empty_env_path("XDG_DATA_HOME") {
        return Ok(path);
    }
    Ok(user_home_dir()?.join(".local").join("share"))
}

pub fn opencode_config_dir() -> Result<PathBuf, String> {
    Ok(xdg_config_home()?.join(OPENCODE_DIR_NAME))
}

pub fn opencode_config_dirs() -> Result<Vec<PathBuf>, String> {
    let mut dirs = Vec::new();
    let mut seen = std::collections::HashSet::new();

    if let Some(dir) = non_empty_env_path("OPENCODE_CONFIG_DIR") {
        push_unique(&mut dirs, &mut seen, dir);
    }

    push_unique(&mut dirs, &mut seen, opencode_config_dir()?);

    Ok(dirs)
}

pub fn opencode_config_file() -> Result<PathBuf, String> {
    Ok(opencode_config_dir()?.join("opencode.json"))
}

pub fn opencode_config_write_file() -> Result<PathBuf, String> {
    for path in opencode_config_candidates()? {
        if path.exists() {
            return Ok(path);
        }
    }

    Ok(opencode_config_file()?)
}

pub fn opencode_config_candidates() -> Result<Vec<PathBuf>, String> {
    let mut candidates = Vec::new();
    let mut seen = std::collections::HashSet::new();

    if let Some(path) = non_empty_env_path("OPENCODE_CONFIG") {
        push_unique(&mut candidates, &mut seen, path);
    }

    for dir in opencode_config_dirs()? {
        push_unique(&mut candidates, &mut seen, dir.join("opencode.json"));
        push_unique(&mut candidates, &mut seen, dir.join("opencode.jsonc"));
    }

    Ok(candidates)
}

pub fn opencode_auth_file() -> Result<PathBuf, String> {
    Ok(xdg_data_home()?.join(OPENCODE_DIR_NAME).join("auth.json"))
}

pub fn omo_cache_dir() -> Result<PathBuf, String> {
    Ok(xdg_cache_home()?.join(OMO_CACHE_DIR_NAME))
}

pub fn omo_switch_config_dir() -> Result<PathBuf, String> {
    Ok(xdg_config_home()?.join(OMO_SWITCH_DIR_NAME))
}

pub fn omo_switch_presets_dir() -> Result<PathBuf, String> {
    Ok(omo_switch_config_dir()?.join("presets"))
}

pub fn opencode_backups_dir() -> Result<PathBuf, String> {
    Ok(opencode_config_dir()?.join("backups"))
}

pub fn parse_json_or_jsonc(content: &str, error_context: &str) -> Result<serde_json::Value, String> {
    serde_json::from_str::<serde_json::Value>(content)
        .or_else(|_| json5::from_str::<serde_json::Value>(content))
        .map_err(|e| format!("{}: {}", error_context, e))
}

pub fn non_empty_env_path(name: &str) -> Option<PathBuf> {
    std::env::var(name)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}

fn push_unique(
    candidates: &mut Vec<PathBuf>,
    seen: &mut std::collections::HashSet<String>,
    path: PathBuf,
) {
    let key = path.to_string_lossy().to_string();
    if seen.insert(key) {
        candidates.push(path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    #[serial]
    fn user_home_dir_uses_userprofile_when_home_missing() {
        let temp_dir = std::env::temp_dir().join("omo-path-service-userprofile-test");
        let original_home = std::env::var("HOME").ok();
        let original_userprofile = std::env::var("USERPROFILE").ok();
        unsafe {
            std::env::remove_var("HOME");
            std::env::set_var("USERPROFILE", &temp_dir);
        }

        let home = user_home_dir().unwrap();

        assert_eq!(home, temp_dir);

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
    }
}
