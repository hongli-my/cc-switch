//! Pi Agent 配置管理模块
//!
//! 负责读取和写入 Pi Agent 的配置文件，包括：
//! - auth.json: API 密钥和认证信息
//! - models.json: 自定义提供商和模型
//! - sessions/: 会话记录（用于用量统计）
//! - skills/: Skill 管理

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;

use crate::error::AppError;

/// Pi 配置文件目录
pub fn get_pi_config_dir() -> PathBuf {
    if let Some(override_dir) = crate::settings::get_pi_override_dir() {
        return override_dir;
    }

    dirs::home_dir()
        .expect("无法获取用户主目录")
        .join(".pi")
        .join("agent")
}

/// Pi auth.json 路径
pub fn get_pi_auth_path() -> PathBuf {
    get_pi_config_dir().join("auth.json")
}

/// Pi models.json 路径
pub fn get_pi_models_path() -> PathBuf {
    get_pi_config_dir().join("models.json")
}

/// Pi 会话目录
pub fn get_pi_sessions_dir() -> PathBuf {
    get_pi_config_dir().join("sessions")
}

/// Pi Skill 目录
pub fn get_pi_skills_dir() -> PathBuf {
    get_pi_config_dir().join("skills")
}

/// Pi 配置结构（auth.json）
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PiAuthConfig {
    #[serde(flatten)]
    pub providers: serde_json::Map<String, Value>,
}

/// Pi 供应商配置（用于 CC Switch 数据库）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PiProviderConfig {
    /// API Key
    pub api_key: Option<String>,
    /// Base URL（自定义 API 端点）
    pub base_url: Option<String>,
    /// 提供商类型（对应 auth.json 中的 key）
    pub provider_type: Option<String>,
    /// 环境变量
    pub env: Option<serde_json::Map<String, Value>>,
}

impl Default for PiProviderConfig {
    fn default() -> Self {
        Self {
            api_key: None,
            base_url: None,
            provider_type: None,
            env: None,
        }
    }
}

/// 读取 Pi auth.json
pub fn read_pi_auth() -> Result<PiAuthConfig, String> {
    let path = get_pi_auth_path();
    if !path.exists() {
        return Ok(PiAuthConfig::default());
    }

    let content = fs::read_to_string(&path)
        .map_err(|e| format!("读取 Pi auth.json 失败: {}", e))?;
    
    serde_json::from_str(&content)
        .map_err(|e| format!("解析 Pi auth.json 失败: {}", e))
}

/// 写入 Pi auth.json
pub fn write_pi_auth(config: &PiAuthConfig) -> Result<(), String> {
    let path = get_pi_auth_path();
    
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("创建 Pi 配置目录失败: {}", e))?;
    }

    let content = serde_json::to_string_pretty(config)
        .map_err(|e| format!("序列化 Pi auth.json 失败: {}", e))?;
    
    fs::write(&path, content)
        .map_err(|e| format!("写入 Pi auth.json 失败: {}", e))?;
    
    // 设置文件权限为 600
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let metadata = fs::metadata(&path)
            .map_err(|e| format!("获取 Pi auth.json 权限失败: {}", e))?;
        let mut permissions = metadata.permissions();
        permissions.set_mode(0o600);
        fs::set_permissions(&path, permissions)
            .map_err(|e| format!("设置 Pi auth.json 权限失败: {}", e))?;
    }

    Ok(())
}

/// 从 Pi auth.json 导入供应商
pub fn import_providers_from_pi_auth() -> Result<Vec<(String, PiProviderConfig)>, String> {
    let auth = read_pi_auth()?;
    let mut providers = Vec::new();

    for (provider_key, value) in auth.providers.iter() {
        if let Value::Object(obj) = value {
            let api_key = obj.get("key")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            
            let provider_type = obj.get("type")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            let config = PiProviderConfig {
                api_key,
                base_url: None,
                provider_type: Some(provider_key.clone()),
                env: None,
            };

            providers.push((provider_key.clone(), config));
        }
    }

    Ok(providers)
}

/// 读取 Pi models.json（返回完整 JSON Value）
///
/// 文件不存在时返回空对象 `{}`。
pub fn read_pi_models() -> Result<Value, AppError> {
    let path = get_pi_models_path();
    if !path.exists() {
        return Ok(json!({}));
    }

    let content = fs::read_to_string(&path).map_err(|e| AppError::io(&path, e))?;
    serde_json::from_str(&content).map_err(|e| AppError::json(&path, e))
}

/// 写入 Pi models.json（原子写）
fn write_pi_models(config: &Value) -> Result<(), AppError> {
    let path = get_pi_models_path();
    crate::config::write_json_file(&path, config)?;
    Ok(())
}

/// 获取 Pi models.json 中的 `providers` 段（返回 map）
///
/// 文件不存在或无 `providers` 段时返回空 map。
pub fn get_providers() -> Result<serde_json::Map<String, Value>, AppError> {
    let config = read_pi_models()?;
    Ok(config
        .get("providers")
        .and_then(|v| v.as_object())
        .cloned()
        .unwrap_or_default())
}

/// 设置（插入或覆盖）Pi models.json 中的一个 provider
pub fn set_provider(id: &str, config: Value) -> Result<(), AppError> {
    let mut full_config = read_pi_models()?;

    if full_config.get("providers").is_none() {
        full_config["providers"] = json!({});
    }

    if let Some(providers) = full_config
        .get_mut("providers")
        .and_then(|v| v.as_object_mut())
    {
        providers.insert(id.to_string(), config);
    }

    write_pi_models(&full_config)
}

/// 从 Pi models.json 中移除一个 provider
pub fn remove_provider(id: &str) -> Result<(), AppError> {
    let mut config = read_pi_models()?;

    if let Some(providers) = config
        .get_mut("providers")
        .and_then(|v| v.as_object_mut())
    {
        providers.remove(id);
    }

    write_pi_models(&config)
}

/// Pi 用量统计
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PiUsageStats {
    pub total_tokens: u64,
    pub session_count: u32,
    pub last_active: Option<u64>,
}

/// 获取 Pi 的用量统计（从会话文件解析）
pub fn get_pi_usage_stats() -> Result<PiUsageStats, String> {
    let sessions_dir = get_pi_sessions_dir();
    if !sessions_dir.exists() {
        return Ok(PiUsageStats::default());
    }

    let mut total_tokens = 0u64;
    let mut session_count = 0u32;
    let mut last_active = None;

    // 遍历会话目录
    for entry in fs::read_dir(&sessions_dir)
        .map_err(|e| format!("读取 Pi 会话目录失败: {}", e))?
    {
        let entry = entry.map_err(|e| format!("读取会话条目失败: {}", e))?;
        let path = entry.path();

        if path.is_dir() {
            session_count += 1;
            
            // 尝试读取会话元数据
            let metadata_path = path.join("metadata.json");
            if metadata_path.exists() {
                if let Ok(content) = fs::read_to_string(&metadata_path) {
                    if let Ok(metadata) = serde_json::from_str::<Value>(&content) {
                        if let Some(tokens) = metadata.get("total_tokens").and_then(|v| v.as_u64()) {
                            total_tokens += tokens;
                        }
                        if let Some(timestamp) = metadata.get("last_active").and_then(|v| v.as_u64()) {
                            if last_active.is_none() || last_active.unwrap() < timestamp {
                                last_active = Some(timestamp);
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(PiUsageStats {
        total_tokens,
        session_count,
        last_active,
    })
}

/// Pi Skill 信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PiSkillInfo {
    pub name: String,
    pub path: String,
    pub description: Option<String>,
    pub enabled: bool,
}

/// 扫描 Pi 的 Skill 目录
pub fn scan_pi_skills() -> Result<Vec<PiSkillInfo>, String> {
    let skills_dir = get_pi_skills_dir();
    if !skills_dir.exists() {
        return Ok(Vec::new());
    }

    let mut skills = Vec::new();

    for entry in fs::read_dir(&skills_dir)
        .map_err(|e| format!("读取 Pi Skill 目录失败: {}", e))?
    {
        let entry = entry.map_err(|e| format!("读取 Skill 条目失败: {}", e))?;
        let path = entry.path();

        if path.is_dir() {
            let name = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string();

            let mut description = None;

            // 尝试读取 SKILL.md
            let skill_md_path = path.join("SKILL.md");
            if skill_md_path.exists() {
                if let Ok(content) = fs::read_to_string(&skill_md_path) {
                    if let Some(first_line) = content.lines().next() {
                        description = Some(first_line.trim_start_matches('#').trim().to_string());
                    }
                }
            }

            skills.push(PiSkillInfo {
                name,
                path: path.to_string_lossy().to_string(),
                description,
                enabled: true,
            });
        }
    }

    Ok(skills)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_pi_config_dir() {
        let dir = get_pi_config_dir();
        assert!(dir.to_string_lossy().contains(".pi"));
        assert!(dir.to_string_lossy().contains("agent"));
    }

    #[test]
    fn test_pi_provider_config_default() {
        let config = PiProviderConfig::default();
        assert!(config.api_key.is_none());
        assert!(config.base_url.is_none());
    }
}
