//! Pi MCP 同步和导入模块
//!
//! Pi Agent 目前没有独立的 MCP 配置段（其 `models.json` 只承载 provider/模型
//! 定义）。本模块提供与其他 additive 应用对称的 API，但实际读写为空操作，
//! 保证 cc-switch 统一 MCP 管理在启用 Pi 时不会报错。
//!
//! 当 Pi 未来引入独立 MCP 配置时，只需在本模块内补齐读写逻辑即可。

use serde_json::Value;

use crate::app_config::MultiAppConfig;
use crate::error::AppError;

// ============================================================================
// Public API: Sync Functions
// ============================================================================

/// Sync a single MCP server to Pi live config
///
/// Pi 暂无独立 MCP 配置文件，这里直接返回 Ok(())。
pub fn sync_single_server_to_pi(
    _config: &MultiAppConfig,
    _id: &str,
    _server_spec: &Value,
) -> Result<(), AppError> {
    Ok(())
}

/// Remove a single MCP server from Pi live config
///
/// Pi 暂无独立 MCP 配置文件，这里直接返回 Ok(())。
pub fn remove_server_from_pi(_id: &str) -> Result<(), AppError> {
    Ok(())
}

/// Import MCP servers from Pi config to unified structure
///
/// Pi 暂无独立 MCP 配置段，导入恒为 0。
pub fn import_from_pi(_config: &mut MultiAppConfig) -> Result<usize, AppError> {
    Ok(0)
}
