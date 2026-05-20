use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashSet;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

const OPENCODE_PACKAGE_NAMES: [&str; 2] = ["opencode-ai", "opencode"];
const OMO_PLUGIN_NAMES: [&str; 2] = ["oh-my-openagent", "oh-my-opencode"];
const OMO_PACKAGE_NAMES: [&str; 2] = ["oh-my-openagent", "oh-my-opencode"];
const OMO_UPDATE_PACKAGE_NAME: &str = "oh-my-opencode";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VersionInfo {
    pub name: String,
    pub current_version: Option<String>,
    pub latest_version: Option<String>,
    pub has_update: bool,
    pub update_command: String,
    pub update_hint: String,
    pub installed: bool,
    pub install_source: Option<String>,
    pub install_path: Option<String>,
    pub detected_from: Option<String>,
}

#[derive(Debug, Clone)]
struct InstallDetection {
    version: Option<String>,
    install_source: String,
    install_path: String,
    detected_from: String,
}

#[derive(Debug, Clone)]
struct DeclaredPlugin {
    plugin_name: String,
    version_spec: Option<String>,
    config_path: String,
}

#[derive(Debug, Clone)]
struct BinaryCandidate {
    command: String,
    install_source: String,
    install_path: String,
}

fn non_empty_env_path(key: &str) -> Option<PathBuf> {
    std::env::var_os(key).and_then(|value| {
        if value.is_empty() {
            None
        } else {
            Some(PathBuf::from(value))
        }
    })
}

fn user_home_dir() -> Option<PathBuf> {
    if cfg!(windows) {
        non_empty_env_path("USERPROFILE")
            .or_else(|| non_empty_env_path("HOME"))
            .or_else(dirs::home_dir)
    } else {
        non_empty_env_path("HOME")
            .or_else(dirs::home_dir)
            .or_else(|| non_empty_env_path("USERPROFILE"))
    }
}

fn path_to_string(path: &Path) -> String {
    path.to_string_lossy().to_string()
}

fn push_unique_path(paths: &mut Vec<PathBuf>, seen: &mut HashSet<OsString>, path: PathBuf) {
    if seen.insert(path.as_os_str().to_os_string()) {
        paths.push(path);
    }
}

fn push_unique_candidate(
    candidates: &mut Vec<BinaryCandidate>,
    seen: &mut HashSet<String>,
    command: String,
    install_source: &str,
    install_path: String,
) {
    if seen.insert(command.clone()) {
        candidates.push(BinaryCandidate {
            command,
            install_source: install_source.to_string(),
            install_path,
        });
    }
}

fn command_names(base: &str) -> Vec<String> {
    if cfg!(windows) {
        vec![
            format!("{}.exe", base),
            format!("{}.cmd", base),
            format!("{}.bat", base),
            base.to_string(),
        ]
    } else {
        vec![base.to_string()]
    }
}

fn push_binary_candidates_from_dir(
    candidates: &mut Vec<BinaryCandidate>,
    seen: &mut HashSet<String>,
    dir: &Path,
    base: &str,
    install_source: &str,
) {
    for name in command_names(base) {
        let binary = dir.join(name);
        if binary.exists() {
            push_unique_candidate(
                candidates,
                seen,
                path_to_string(&binary),
                install_source,
                path_to_string(dir),
            );
        }
    }
}

fn common_binary_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    let mut seen = HashSet::new();

    if let Some(home) = user_home_dir() {
        push_unique_path(&mut dirs, &mut seen, home.join(".opencode").join("bin"));
        push_unique_path(&mut dirs, &mut seen, home.join(".bun").join("bin"));
        push_unique_path(&mut dirs, &mut seen, home.join(".local").join("bin"));
        push_unique_path(&mut dirs, &mut seen, home.join("scoop").join("shims"));

        if cfg!(windows) {
            push_unique_path(
                &mut dirs,
                &mut seen,
                home.join("AppData").join("Roaming").join("npm"),
            );
            push_unique_path(
                &mut dirs,
                &mut seen,
                home.join("AppData").join("Local").join("pnpm"),
            );
        }
    }

    if let Some(appdata) = non_empty_env_path("APPDATA") {
        push_unique_path(&mut dirs, &mut seen, appdata.join("npm"));
    }
    if let Some(localappdata) = non_empty_env_path("LOCALAPPDATA") {
        push_unique_path(&mut dirs, &mut seen, localappdata.join("pnpm"));
        push_unique_path(
            &mut dirs,
            &mut seen,
            localappdata.join("Microsoft").join("WinGet").join("Links"),
        );
    }
    if let Some(pnpm_home) = non_empty_env_path("PNPM_HOME") {
        push_unique_path(&mut dirs, &mut seen, pnpm_home);
    }
    if let Some(bun_install) = non_empty_env_path("BUN_INSTALL") {
        push_unique_path(&mut dirs, &mut seen, bun_install.join("bin"));
    }
    if let Some(program_files) = non_empty_env_path("ProgramFiles") {
        push_unique_path(&mut dirs, &mut seen, program_files.join("nodejs"));
    }
    if let Some(program_files_x86) = non_empty_env_path("ProgramFiles(x86)") {
        push_unique_path(&mut dirs, &mut seen, program_files_x86.join("nodejs"));
    }

    dirs
}

fn get_package_roots() -> Vec<PathBuf> {
    get_package_roots_for_home(user_home_dir().as_deref())
}

fn get_package_roots_for_home(home: Option<&Path>) -> Vec<PathBuf> {
    let mut roots = Vec::new();
    let mut seen = HashSet::new();

    if let Some(home) = home {
        push_unique_path(
            &mut roots,
            &mut seen,
            home.join(".opencode").join("node_modules"),
        );
        push_unique_path(
            &mut roots,
            &mut seen,
            home.join(".config").join("opencode").join("node_modules"),
        );
        push_unique_path(
            &mut roots,
            &mut seen,
            home.join(".cache").join("opencode").join("node_modules"),
        );
        push_unique_path(
            &mut roots,
            &mut seen,
            home.join(".bun")
                .join("install")
                .join("global")
                .join("node_modules"),
        );

        if cfg!(windows) {
            push_unique_path(
                &mut roots,
                &mut seen,
                home.join("AppData")
                    .join("Roaming")
                    .join("npm")
                    .join("node_modules"),
            );
        }
    }

    if let Some(appdata) = non_empty_env_path("APPDATA") {
        push_unique_path(
            &mut roots,
            &mut seen,
            appdata.join("npm").join("node_modules"),
        );
    }
    if let Some(npm_root) = get_npm_global_root() {
        push_unique_path(&mut roots, &mut seen, PathBuf::from(npm_root));
    }

    roots
}

fn get_opencode_package_root_for_spec(
    home_dir: &Path,
    plugin_name: &str,
    version_spec: &str,
) -> PathBuf {
    home_dir
        .join(".cache")
        .join("opencode")
        .join("packages")
        .join(format!("{}@{}", plugin_name, version_spec))
        .join("node_modules")
}

fn cache_package_spec(version_spec: &str) -> Option<&str> {
    let trimmed = version_spec.trim();
    if trimmed.is_empty() {
        return None;
    }

    if trimmed == "latest" || trimmed.chars().next().is_some_and(|ch| ch.is_ascii_digit()) {
        Some(trimmed)
    } else {
        None
    }
}

fn declared_version_value(version_spec: Option<String>) -> Option<String> {
    match version_spec {
        Some(spec) if spec.trim().is_empty() || spec.trim() == "latest" => None,
        other => other,
    }
}

fn build_opencode_binary_candidates() -> Vec<BinaryCandidate> {
    let mut candidates = Vec::new();
    let mut seen = HashSet::new();

    if let Ok(path) = std::env::var("OPENCODE_BIN") {
        let trimmed = path.trim();
        if !trimmed.is_empty() {
            push_unique_candidate(
                &mut candidates,
                &mut seen,
                trimmed.to_string(),
                "opencode_bin_env",
                trimmed.to_string(),
            );
        }
    }

    extend_opencode_binary_candidates(&mut candidates, &mut seen);

    candidates
}

#[cfg(test)]
fn build_opencode_binary_candidates_with_explicit_binary(command: &Path) -> Vec<BinaryCandidate> {
    let mut candidates = Vec::new();
    let mut seen = HashSet::new();
    let command = path_to_string(command);
    push_unique_candidate(
        &mut candidates,
        &mut seen,
        command.clone(),
        "opencode_bin_env",
        command,
    );

    candidates
}

fn extend_opencode_binary_candidates(
    candidates: &mut Vec<BinaryCandidate>,
    seen: &mut HashSet<String>,
) {
    for dir in common_binary_dirs() {
        push_binary_candidates_from_dir(candidates, seen, &dir, "opencode", "known_binary_dir");
    }

    push_unique_candidate(
        candidates,
        seen,
        "opencode".to_string(),
        "path",
        "opencode".to_string(),
    );
}

fn run_version_command(command: &str) -> Option<String> {
    let mut child = Command::new(command)
        .arg("--version")
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;

    let timeout = Duration::from_secs(3);
    let start = Instant::now();

    loop {
        match child.try_wait() {
            Ok(Some(status)) if status.success() => {
                let output = child.wait_with_output().ok()?;
                let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
                return if !version.is_empty() {
                    Some(version)
                } else {
                    None
                };
            }
            Ok(Some(_)) => return None,
            Ok(None) => {
                if start.elapsed() > timeout {
                    let _ = child.kill();
                    let _ = child.wait();
                    return None;
                }
                std::thread::sleep(Duration::from_millis(100));
            }
            Err(_) => return None,
        }
    }
}

fn detect_opencode_install() -> Option<InstallDetection> {
    detect_opencode_install_from_candidates(build_opencode_binary_candidates(), get_package_roots())
}

fn detect_opencode_install_from_candidates(
    candidates: Vec<BinaryCandidate>,
    package_roots: Vec<PathBuf>,
) -> Option<InstallDetection> {
    for candidate in candidates {
        if let Some(version) = run_version_command(&candidate.command) {
            return Some(InstallDetection {
                version: Some(version),
                install_source: candidate.install_source,
                install_path: candidate.install_path,
                detected_from: candidate.command,
            });
        }
    }

    for root in package_roots {
        for package_name in OPENCODE_PACKAGE_NAMES {
            let pkg = root.join(package_name).join("package.json");
            if let Some(version) = read_pkg_version_path(&pkg) {
                return Some(InstallDetection {
                    version: Some(version),
                    install_source: "package_root".to_string(),
                    install_path: path_to_string(&root),
                    detected_from: path_to_string(&pkg),
                });
            }
        }
    }

    None
}

fn detect_omo_install() -> Option<InstallDetection> {
    let home_dir = user_home_dir()?;
    detect_omo_install_for_home(
        &home_dir,
        get_opencode_config_candidates(&path_to_string(&home_dir)),
    )
}

fn detect_omo_install_for_home(
    home_dir: &Path,
    config_candidates: Vec<String>,
) -> Option<InstallDetection> {
    let home = path_to_string(home_dir);

    let declared_plugin = config_candidates
        .iter()
        .find_map(|config_path| read_declared_plugin(config_path, &OMO_PLUGIN_NAMES));

    if let Some(declared) = declared_plugin.as_ref() {
        if let Some(version_spec) = declared
            .version_spec
            .as_deref()
            .and_then(cache_package_spec)
        {
            let package_root =
                get_opencode_package_root_for_spec(home_dir, &declared.plugin_name, version_spec);
            let package_name = if declared.plugin_name == "oh-my-opencode" {
                "oh-my-opencode"
            } else {
                "oh-my-openagent"
            };
            let pkg = package_root.join(package_name).join("package.json");
            if let Some(version) = read_pkg_version_path(&pkg) {
                return Some(InstallDetection {
                    version: Some(version),
                    install_source: "opencode_package_cache".to_string(),
                    install_path: path_to_string(&package_root),
                    detected_from: path_to_string(&pkg),
                });
            }
        }
    }

    // 1. 当前实际 opencode 运行目录: ~/.opencode/node_modules/<omo-package>/
    for package_name in OMO_PACKAGE_NAMES {
        let runtime_pkg = format!(
            "{}/.opencode/node_modules/{}/package.json",
            home, package_name
        );
        if let Some(version) = read_pkg_version(&runtime_pkg) {
            return Some(InstallDetection {
                version: Some(version),
                install_source: "opencode_runtime".to_string(),
                install_path: format!("{}/.opencode", home),
                detected_from: runtime_pkg,
            });
        }
    }

    // 2. 当前实际 opencode 运行目录依赖声明: ~/.opencode/package.json
    let runtime_dep_pkg = format!("{}/.opencode/package.json", home);
    for package_name in OMO_PACKAGE_NAMES {
        if let Some(version) = read_dependency_version(&runtime_dep_pkg, package_name) {
            return Some(InstallDetection {
                version: Some(version),
                install_source: "opencode_runtime".to_string(),
                install_path: format!("{}/.opencode", home),
                detected_from: runtime_dep_pkg.clone(),
            });
        }
    }

    // 3. 本地安装: ~/.config/opencode/node_modules/<omo-package>/
    for package_name in OMO_PACKAGE_NAMES {
        let local_pkg = format!(
            "{}/.config/opencode/node_modules/{}/package.json",
            home, package_name
        );
        if let Some(version) = read_pkg_version(&local_pkg) {
            return Some(InstallDetection {
                version: Some(version),
                install_source: "config_local".to_string(),
                install_path: format!("{}/.config/opencode", home),
                detected_from: local_pkg,
            });
        }
    }

    // 4. 配置文件: opencode.json/jsonc 的 plugin 字段（兼容 openagent/opencode 插件名）
    if let Some(declared) = declared_plugin {
        return Some(InstallDetection {
            version: declared_version_value(declared.version_spec),
            install_source: "config_declared".to_string(),
            install_path: declared.config_path.clone(),
            detected_from: declared.config_path,
        });
    }

    // 5. npm 全局安装
    if let Some(global_root) = get_npm_global_root() {
        for package_name in OMO_PACKAGE_NAMES {
            let npm_global_pkg = format!("{}/{}/package.json", global_root, package_name);
            if let Some(version) = read_pkg_version(&npm_global_pkg) {
                return Some(InstallDetection {
                    version: Some(version),
                    install_source: "npm_global".to_string(),
                    install_path: global_root.clone(),
                    detected_from: npm_global_pkg,
                });
            }
        }
    }

    for root in get_package_roots_for_home(Some(home_dir)) {
        for package_name in OMO_PACKAGE_NAMES {
            let pkg = root.join(package_name).join("package.json");
            if let Some(version) = read_pkg_version_path(&pkg) {
                return Some(InstallDetection {
                    version: Some(version),
                    install_source: "package_root".to_string(),
                    install_path: path_to_string(&root),
                    detected_from: path_to_string(&pkg),
                });
            }
        }
    }

    // 6. bun 全局安装: ~/.bun/install/global/node_modules/<omo-package>/
    for package_name in OMO_PACKAGE_NAMES {
        let bun_global = format!(
            "{}/.bun/install/global/node_modules/{}/package.json",
            home, package_name
        );
        if let Some(version) = read_pkg_version(&bun_global) {
            return Some(InstallDetection {
                version: Some(version),
                install_source: "bun_global".to_string(),
                install_path: format!("{}/.bun/install/global/node_modules", home),
                detected_from: bun_global,
            });
        }
    }

    // 7. opencode 缓存安装/依赖，作为最后回退
    for package_name in OMO_PACKAGE_NAMES {
        let cache_pkg = format!(
            "{}/.cache/opencode/node_modules/{}/package.json",
            home, package_name
        );
        if let Some(version) = read_pkg_version(&cache_pkg) {
            return Some(InstallDetection {
                version: Some(version),
                install_source: "opencode_cache".to_string(),
                install_path: format!("{}/.cache/opencode", home),
                detected_from: cache_pkg,
            });
        }
    }

    let cache_dep_pkg = format!("{}/.cache/opencode/package.json", home);
    for package_name in OMO_PACKAGE_NAMES {
        if let Some(version) = read_dependency_version(&cache_dep_pkg, package_name) {
            return Some(InstallDetection {
                version: Some(version),
                install_source: "opencode_cache".to_string(),
                install_path: format!("{}/.cache/opencode", home),
                detected_from: cache_dep_pkg.clone(),
            });
        }
    }

    None
}

fn read_pkg_version(path: &str) -> Option<String> {
    let content = std::fs::read_to_string(path).ok()?;
    let pkg: serde_json::Value = serde_json::from_str(&content).ok()?;
    pkg.get("version")?.as_str().map(|s| s.to_string())
}

fn read_pkg_version_path(path: &Path) -> Option<String> {
    let content = std::fs::read_to_string(path).ok()?;
    let pkg: serde_json::Value = serde_json::from_str(&content).ok()?;
    pkg.get("version")?.as_str().map(|s| s.to_string())
}

fn read_dependency_version(path: &str, dep_name: &str) -> Option<String> {
    let content = std::fs::read_to_string(path).ok()?;
    let pkg: Value = serde_json::from_str(&content).ok()?;
    pkg.get("dependencies")?
        .get(dep_name)?
        .as_str()
        .map(|s| s.to_string())
}

fn get_opencode_config_candidates(home: &str) -> Vec<String> {
    let mut candidates = Vec::new();
    let mut seen = HashSet::new();

    if let Some(config_path) = non_empty_env_path("OPENCODE_CONFIG") {
        push_unique_path(&mut candidates, &mut seen, config_path);
    }

    if let Some(config_dir) = non_empty_env_path("OPENCODE_CONFIG_DIR") {
        push_unique_path(&mut candidates, &mut seen, config_dir.join("opencode.json"));
        push_unique_path(
            &mut candidates,
            &mut seen,
            config_dir.join("opencode.jsonc"),
        );
    }

    let default_config_dir = PathBuf::from(home).join(".config").join("opencode");
    push_unique_path(
        &mut candidates,
        &mut seen,
        default_config_dir.join("opencode.json"),
    );
    push_unique_path(
        &mut candidates,
        &mut seen,
        default_config_dir.join("opencode.jsonc"),
    );

    candidates
        .into_iter()
        .map(|path| path_to_string(&path))
        .collect()
}

fn parse_json_or_json5(content: &str) -> Option<Value> {
    serde_json::from_str::<Value>(content)
        .or_else(|_| json5::from_str::<Value>(content))
        .ok()
}

fn read_declared_plugin(path: &str, plugin_names: &[&str]) -> Option<DeclaredPlugin> {
    let content = std::fs::read_to_string(path).ok()?;
    let config = parse_json_or_json5(&content)?;
    let plugins = config.get("plugin")?.as_array()?;

    for plugin in plugins {
        let raw = plugin.as_str()?.trim();
        for plugin_name in plugin_names {
            if raw == *plugin_name {
                return Some(DeclaredPlugin {
                    plugin_name: plugin_name.to_string(),
                    version_spec: None,
                    config_path: path.to_string(),
                });
            }

            if let Some(version_spec) = raw.strip_prefix(&format!("{}@", plugin_name)) {
                return Some(DeclaredPlugin {
                    plugin_name: plugin_name.to_string(),
                    version_spec: Some(version_spec.to_string()),
                    config_path: path.to_string(),
                });
            }
        }
    }

    None
}

fn get_npm_global_root() -> Option<String> {
    let output = Command::new("npm")
        .args(["root", "-g"])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let root = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if root.is_empty() {
        None
    } else {
        Some(root)
    }
}

fn is_omo_installed() -> bool {
    if detect_omo_install().is_some() {
        return true;
    }

    let home = match user_home_dir() {
        Some(home) => path_to_string(&home),
        None => return false,
    };
    get_opencode_config_candidates(&home)
        .iter()
        .any(|path| is_plugin_declared_in_config(path, &OMO_PLUGIN_NAMES))
}

fn build_omo_update_command(install_source: Option<&str>) -> (String, String) {
    match install_source {
        Some("opencode_runtime") => (
            format!(
                "cd ~/.opencode && npm install {}@latest",
                OMO_UPDATE_PACKAGE_NAME
            ),
            "在当前 opencode 运行目录升级：".to_string(),
        ),
        Some("npm_global") => (
            format!("npm install -g {}@latest", OMO_UPDATE_PACKAGE_NAME),
            "通过 npm 全局升级：".to_string(),
        ),
        Some("bun_global") => (
            format!("bun add -g {}@latest", OMO_UPDATE_PACKAGE_NAME),
            "通过 bun 全局升级：".to_string(),
        ),
        Some("config_local") => (
            format!(
                "cd ~/.config/opencode && npm install {}@latest",
                OMO_UPDATE_PACKAGE_NAME
            ),
            "在本地 opencode 配置目录升级：".to_string(),
        ),
        Some("config_declared") => (
            format!(
                "cd ~/.opencode && npm install {}@latest",
                OMO_UPDATE_PACKAGE_NAME
            ),
            "配置中已声明插件，建议在实际运行目录安装/升级：".to_string(),
        ),
        Some("opencode_cache") => (
            format!(
                "cd ~/.opencode && npm install {}@latest",
                OMO_UPDATE_PACKAGE_NAME
            ),
            "检测到缓存版本，建议在实际运行目录重新安装：".to_string(),
        ),
        _ => (
            format!(
                "cd ~/.opencode && npm install {}@latest",
                OMO_UPDATE_PACKAGE_NAME
            ),
            "建议在 opencode 运行目录安装/升级：".to_string(),
        ),
    }
}

fn is_plugin_declared_in_config(path: &str, plugin_names: &[&str]) -> bool {
    let content = match std::fs::read_to_string(path) {
        Ok(content) => content,
        Err(_) => return false,
    };
    let config = match parse_json_or_json5(&content) {
        Some(config) => config,
        None => return false,
    };
    let plugins = match config.get("plugin").and_then(|v| v.as_array()) {
        Some(plugins) => plugins,
        None => return false,
    };

    plugins.iter().any(|plugin| {
        let Some(raw) = plugin.as_str() else {
            return false;
        };
        plugin_names
            .iter()
            .any(|name| raw == *name || raw.starts_with(&format!("{}@", name)))
    })
}

fn get_npm_latest_version(package_name: &str) -> Option<String> {
    let url = format!("https://registry.npmjs.org/{}/latest", package_name);
    let resp = ureq::get(&url)
        .timeout(std::time::Duration::from_secs(4))
        .call()
        .ok()?;
    let json: serde_json::Value = resp.into_json().ok()?;
    json.get("version")?.as_str().map(|s| s.to_string())
}

/// Get Oh My OpenAgent latest version from npm registry (兼容旧包名)
pub fn get_omo_latest_version() -> Option<String> {
    get_npm_latest_version("oh-my-openagent").or_else(|| get_npm_latest_version("oh-my-opencode"))
}

/// Get OpenCode latest version from GitHub Releases
pub fn get_opencode_latest_version() -> Option<String> {
    let resp = ureq::get("https://api.github.com/repos/anomalyco/opencode/releases/latest")
        .set("User-Agent", "OMO-Switch")
        .timeout(std::time::Duration::from_secs(3))
        .call()
        .ok()?;
    let json: serde_json::Value = resp.into_json().ok()?;
    json.get("tag_name")?
        .as_str()
        .map(|s| s.trim_start_matches('v').to_string())
}

/// Simple semver comparison: returns true if latest > current
pub fn has_newer_version(current: &str, latest: &str) -> bool {
    let parse = |v: &str| -> Vec<u32> { v.split('.').filter_map(|s| s.parse().ok()).collect() };
    let c = parse(current);
    let l = parse(latest);
    l > c
}

/// Check all versions
pub fn check_all_versions() -> Vec<VersionInfo> {
    let mut results = Vec::new();

    // OpenCode
    let opencode_detection = detect_opencode_install();
    let oc_current = opencode_detection.as_ref().and_then(|d| d.version.clone());
    let oc_latest = get_opencode_latest_version();
    results.push(VersionInfo {
        name: "OpenCode".to_string(),
        installed: oc_current.is_some(),
        current_version: oc_current.clone(),
        latest_version: oc_latest.clone(),
        has_update: match (&oc_current, &oc_latest) {
            (Some(c), Some(l)) => has_newer_version(c, l),
            _ => false,
        },
        update_command: "opencode upgrade".to_string(),
        update_hint: "Run 'opencode upgrade' in terminal".to_string(),
        install_source: opencode_detection
            .as_ref()
            .map(|d| d.install_source.clone()),
        install_path: opencode_detection.as_ref().map(|d| d.install_path.clone()),
        detected_from: opencode_detection.as_ref().map(|d| d.detected_from.clone()),
    });

    // Oh My OpenAgent
    let omo_detection = detect_omo_install();
    let omo_current = omo_detection.as_ref().and_then(|d| d.version.clone());
    let omo_latest = get_omo_latest_version();
    let has_update = match (&omo_current, &omo_latest) {
        (Some(c), Some(l)) => has_newer_version(c, l),
        _ => false,
    };
    let (update_command, update_hint) =
        build_omo_update_command(omo_detection.as_ref().map(|d| d.install_source.as_str()));
    results.push(VersionInfo {
        name: "Oh My OpenAgent".to_string(),
        installed: is_omo_installed(),
        current_version: omo_current.clone(),
        latest_version: omo_latest.clone(),
        has_update,
        update_command,
        update_hint,
        install_source: omo_detection.as_ref().map(|d| d.install_source.clone()),
        install_path: omo_detection.as_ref().map(|d| d.install_path.clone()),
        detected_from: omo_detection.as_ref().map(|d| d.detected_from.clone()),
    });

    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::{Path, PathBuf};

    fn fresh_temp_dir(name: &str) -> PathBuf {
        let path = std::env::temp_dir().join(name);
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();
        path
    }

    fn write_mock_opencode(path: &Path) {
        #[cfg(windows)]
        {
            fs::write(path, "@echo 9.8.7\r\n").unwrap();
        }

        #[cfg(not(windows))]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::write(path, "#!/bin/sh\necho 9.8.7\n").unwrap();
            let mut permissions = fs::metadata(path).unwrap().permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(path, permissions).unwrap();
        }
    }

    #[test]
    fn test_has_newer_version() {
        assert!(has_newer_version("3.5.2", "3.5.3"));
        assert!(!has_newer_version("3.5.3", "3.5.3"));
        assert!(!has_newer_version("3.5.3", "3.5.2"));
        assert!(has_newer_version("3.4.0", "3.5.0"));
    }

    #[test]
    fn detects_opencode_from_explicit_binary_when_home_missing() {
        let temp_dir = fresh_temp_dir("omo-version-opencode-bin-test");
        let binary = temp_dir.join(if cfg!(windows) {
            "opencode.cmd"
        } else {
            "opencode"
        });
        write_mock_opencode(&binary);

        let detection = detect_opencode_install_from_candidates(
            build_opencode_binary_candidates_with_explicit_binary(&binary),
            vec![],
        )
        .expect("should detect opencode from explicit binary");
        assert_eq!(detection.version.as_deref(), Some("9.8.7"));
        assert_eq!(detection.install_source, "opencode_bin_env");

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn detects_omo_config_from_userprofile_when_home_missing() {
        let temp_dir = fresh_temp_dir("omo-version-userprofile-test");
        let config_dir = temp_dir.join(".config").join("opencode");
        fs::create_dir_all(&config_dir).unwrap();
        fs::write(
            config_dir.join("opencode.json"),
            r#"{"plugin":["oh-my-openagent@3.17.4"]}"#,
        )
        .unwrap();

        let detection = detect_omo_install_for_home(
            &temp_dir,
            vec![path_to_string(&config_dir.join("opencode.json"))],
        )
        .expect("should detect OMO from USERPROFILE config");
        assert_eq!(detection.version.as_deref(), Some("3.17.4"));
        assert_eq!(detection.install_source, "config_declared");

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn detects_omo_from_userprofile_npm_package_root_when_home_missing() {
        let temp_dir = fresh_temp_dir("omo-version-npm-package-root-test");
        let package_dir = temp_dir
            .join("AppData")
            .join("Roaming")
            .join("npm")
            .join("node_modules")
            .join("oh-my-openagent");
        fs::create_dir_all(&package_dir).unwrap();
        fs::write(package_dir.join("package.json"), r#"{"version":"3.17.4"}"#).unwrap();

        let detection = detect_omo_install_for_home(&temp_dir, vec![])
            .expect("should detect OMO from npm package root");
        assert_eq!(detection.version.as_deref(), Some("3.17.4"));
        assert_eq!(detection.install_source, "package_root");

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn detects_omo_from_opencode_config_dir_override() {
        let temp_dir = fresh_temp_dir("omo-version-config-dir-test");
        let config_dir = temp_dir.join("custom-opencode-config");
        fs::create_dir_all(&config_dir).unwrap();
        fs::write(
            config_dir.join("opencode.jsonc"),
            r#"{"plugin":["oh-my-openagent@3.17.4"]}"#,
        )
        .unwrap();

        let detection = detect_omo_install_for_home(
            &temp_dir,
            vec![path_to_string(&config_dir.join("opencode.jsonc"))],
        )
        .expect("should detect OMO from OPENCODE_CONFIG_DIR");
        assert_eq!(detection.version.as_deref(), Some("3.17.4"));
        assert_eq!(detection.install_source, "config_declared");

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn detects_omo_runtime_package_version_when_config_uses_latest() {
        let temp_dir = fresh_temp_dir("omo-version-latest-package-cache-test");
        let config_dir = temp_dir.join(".config").join("opencode");
        fs::create_dir_all(&config_dir).unwrap();
        fs::write(
            config_dir.join("opencode.json"),
            r#"{"plugin":["oh-my-openagent@latest"]}"#,
        )
        .unwrap();

        let stale_package_dir = temp_dir
            .join(".cache")
            .join("opencode")
            .join("node_modules")
            .join("oh-my-openagent");
        fs::create_dir_all(&stale_package_dir).unwrap();
        fs::write(
            stale_package_dir.join("package.json"),
            r#"{"version":"3.16.0"}"#,
        )
        .unwrap();

        let active_package_dir = temp_dir
            .join(".cache")
            .join("opencode")
            .join("packages")
            .join("oh-my-openagent@latest")
            .join("node_modules")
            .join("oh-my-openagent");
        fs::create_dir_all(&active_package_dir).unwrap();
        fs::write(
            active_package_dir.join("package.json"),
            r#"{"version":"4.2.2"}"#,
        )
        .unwrap();

        let detection = detect_omo_install_for_home(
            &temp_dir,
            vec![path_to_string(&config_dir.join("opencode.json"))],
        )
        .expect("should detect active OMO package version");
        assert_eq!(detection.version.as_deref(), Some("4.2.2"));
        assert_eq!(detection.install_source, "opencode_package_cache");

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn does_not_treat_cache_only_omo_as_installed() {
        let temp_dir = fresh_temp_dir("omo-cache-only-not-installed-test");
        let cache_package_dir = temp_dir
            .join(".cache")
            .join("opencode")
            .join("packages")
            .join("oh-my-openagent@latest")
            .join("node_modules")
            .join("oh-my-openagent");
        fs::create_dir_all(&cache_package_dir).unwrap();
        fs::write(
            cache_package_dir.join("package.json"),
            r#"{"version":"4.2.2"}"#,
        )
        .unwrap();

        let detection = detect_omo_install_for_home(&temp_dir, vec![]);
        assert!(detection.is_none());

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn prefers_pinned_config_over_latest_cache() {
        let temp_dir = fresh_temp_dir("omo-pinned-version-over-cache-test");
        let config_dir = temp_dir.join(".config").join("opencode");
        fs::create_dir_all(&config_dir).unwrap();
        fs::write(
            config_dir.join("opencode.json"),
            r#"{"plugin":["oh-my-openagent@3.17.4"]}"#,
        )
        .unwrap();

        let pinned_package_dir = temp_dir
            .join(".cache")
            .join("opencode")
            .join("packages")
            .join("oh-my-openagent@3.17.4")
            .join("node_modules")
            .join("oh-my-openagent");
        fs::create_dir_all(&pinned_package_dir).unwrap();
        fs::write(
            pinned_package_dir.join("package.json"),
            r#"{"version":"3.17.4"}"#,
        )
        .unwrap();

        let latest_package_dir = temp_dir
            .join(".cache")
            .join("opencode")
            .join("packages")
            .join("oh-my-openagent@latest")
            .join("node_modules")
            .join("oh-my-openagent");
        fs::create_dir_all(&latest_package_dir).unwrap();
        fs::write(
            latest_package_dir.join("package.json"),
            r#"{"version":"4.2.2"}"#,
        )
        .unwrap();

        let detection = detect_omo_install_for_home(
            &temp_dir,
            vec![path_to_string(&config_dir.join("opencode.json"))],
        )
        .expect("should detect pinned OMO package version");
        assert_eq!(detection.version.as_deref(), Some("3.17.4"));
        assert_eq!(detection.install_source, "opencode_package_cache");
        assert!(detection.detected_from.contains("oh-my-openagent@3.17.4"));

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
