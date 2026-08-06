//! Pi Agent 配置管理模块
//!
//! 负责读取和写入 Pi Agent 的配置文件：
//! - models.json: 自定义提供商和模型
//! - sessions/: 会话记录（用于用量统计，由 `session_usage_pi` 解析）
//! - skills/: Skill 管理（由通用 `skill` 服务扫描）

use serde_json::{json, Value};
use std::fs;
use std::path::PathBuf;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_pi_config_dir() {
        let dir = get_pi_config_dir();
        assert!(dir.to_string_lossy().contains(".pi"));
        assert!(dir.to_string_lossy().contains("agent"));
    }
}
