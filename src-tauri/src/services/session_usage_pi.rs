//! Pi 会话日志使用追踪
//!
//! 从 `~/.pi/agent/sessions/` 下所有 `.jsonl`（含主会话与 subagent 会话）中提取精确 token 使用数据。
//!
//! Pi 的会话目录结构：
//! ```text
//! sessions/<cwd-encoded>/
//!   <ts>_<uuid>.jsonl                         ← 主会话
//!   <ts>_<uuid>/<subagent-id>/run-N/session.jsonl  ← subagent 会话
//! ```
//!
//! ## 数据流
//! ```text
//! ~/.pi/agent/sessions/<cwd-encoded>/*.jsonl
//!   → 逐行解析 JSON 事件
//!   → 筛选 type == "message" && message.role == "assistant" && message.usage 存在
//!   → 提取 usage.{input,output,reasoning,cacheRead,cacheWrite} + cost 明细 + model + timestamp
//!   → proxy_request_logs 表 (app_type = "pi")
//! ```
//!
//! ## 与 OpenCode 的差异
//! - OpenCode 从 SQLite (opencode.db) 读；Pi 从 JSONL 文件读。
//! - Pi 的 usage 自带明细 cost（input/output/cacheRead/cacheWrite/total），可直接分项入库，
//!   无需像 OpenCode 那样把聚合 cost 全塞进 total_cost。
//! - Pi 的 timestamp 是 ISO 字符串（`2026-08-05T07-27-34-352Z`，时分秒毫秒以 `-` 分隔），
//!   需预处理成标准 RFC3339 再用 chrono 解析。

use crate::database::{lock_conn, Database};
use crate::error::AppError;
use crate::pi_config::get_pi_sessions_dir;
use crate::proxy::usage::calculator::CostCalculator;
use crate::proxy::usage::parser::TokenUsage;
use crate::services::session_usage::{
    get_sync_state, metadata_modified_nanos, update_sync_state, SessionSyncResult,
};
use crate::services::usage_stats::{find_model_pricing, should_skip_session_insert, DedupKey};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use std::fs;
use std::path::PathBuf;

/// 从 Pi jsonl 一条 assistant 消息提取的 token 和费用数据
struct PiMessageData {
    input_tokens: u32,
    output_tokens: u32,
    reasoning_tokens: u32,
    cache_read_tokens: u32,
    cache_write_tokens: u32,
    /// Pi 自带的明细费用（USD）
    cost_input: f64,
    cost_output: f64,
    cost_cache_read: f64,
    cost_cache_write: f64,
    cost_total: f64,
    model_id: String,
    /// Unix 秒
    created_at: i64,
    /// 幂等键的响应标识（优先 responseId，其次 message.id）
    response_id: String,
}

/// 同步 Pi 使用数据。
///
/// 递归遍历 `~/.pi/agent/sessions/` 下所有 `.jsonl` 文件（主会话 + subagent 会话），
/// 按文件 mtime 做水位线增量：文件未变则跳过；变了则重新解析全部行，
/// 已入库的行靠 `request_id` + `INSERT OR IGNORE` + `DedupKey` 去重。
pub fn sync_pi_usage(db: &Database) -> Result<SessionSyncResult, AppError> {
    let sessions_dir = get_pi_sessions_dir();
    if !sessions_dir.exists() {
        return Ok(SessionSyncResult {
            imported: 0,
            skipped: 0,
            files_scanned: 0,
            errors: vec![],
        });
    }

    let mut result = SessionSyncResult {
        imported: 0,
        skipped: 0,
        files_scanned: 0,
        errors: vec![],
    };
    let mut has_global_error = false;

    // 递归收集所有 .jsonl 文件（主会话 + subagent 会话）
    let mut jsonl_files: Vec<PathBuf> = Vec::new();
    collect_jsonl_files(&sessions_dir, &mut jsonl_files);

    for file_path in jsonl_files {
        result.files_scanned += 1;

        let sync_key = file_path.to_string_lossy().to_string();

        let metadata = match fs::metadata(&file_path) {
            Ok(m) => m,
            Err(e) => {
                let msg = format!("读取 Pi jsonl 元数据失败 {}: {e}", file_path.display());
                log::warn!("[PI-SYNC] {msg}");
                result.errors.push(msg);
                has_global_error = true;
                continue;
            }
        };
        let file_modified = metadata_modified_nanos(&metadata);

        let (last_modified, _) = match get_sync_state(db, &sync_key) {
            Ok(v) => v,
            Err(e) => {
                let msg = format!("读取 Pi 同步状态失败 {}: {e}", file_path.display());
                log::warn!("[PI-SYNC] {msg}");
                result.errors.push(msg);
                has_global_error = true;
                continue;
            }
        };

        // 文件未变化则跳过
        if file_modified <= last_modified {
            continue;
        }

        let content = match fs::read_to_string(&file_path) {
            Ok(c) => c,
            Err(e) => {
                let msg = format!("读取 Pi jsonl 内容失败 {}: {e}", file_path.display());
                log::warn!("[PI-SYNC] {msg}");
                result.errors.push(msg);
                has_global_error = true;
                continue;
            }
        };

        let mut file_had_error = false;

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            let value: serde_json::Value = match serde_json::from_str(line) {
                Ok(v) => v,
                Err(_) => continue, // 跳过无法解析的行（可能是不完整写入）
            };

            if value.get("type").and_then(|t| t.as_str()) != Some("message") {
                continue;
            }

            let message = match value.get("message") {
                Some(m) => m,
                None => continue,
            };

            if message.get("role").and_then(|r| r.as_str()) != Some("assistant") {
                continue;
            }

            if message.get("usage").is_none() {
                continue;
            }

            let msg_data = match parse_pi_message(message, &value) {
                Some(d) => d,
                None => continue,
            };

            // 顶层 id 作为 session 标识；request_id 用 session + response_id 做幂等
            let session_id = value
                .get("id")
                .and_then(|i| i.as_str())
                .unwrap_or("unknown");
            let request_id = format!("pi_session:{session_id}:{}", msg_data.response_id);

            match insert_pi_message(db, &request_id, &msg_data, session_id) {
                Ok(true) => result.imported += 1,
                Ok(false) => result.skipped += 1,
                Err(e) => {
                    let msg = format!("Pi 消息插入失败 {request_id}: {e}");
                    log::warn!("[PI-SYNC] {msg}");
                    result.errors.push(msg);
                    result.skipped += 1;
                    file_had_error = true;
                }
            }
        }

        // 仅当本文件无错误时推进水位线，确保下次可重试
        if file_had_error {
            has_global_error = true;
        } else if let Err(e) = update_sync_state(db, &sync_key, file_modified, 0) {
            let msg = format!("Pi 同步状态更新失败 {}: {e}", file_path.display());
            log::warn!("[PI-SYNC] {msg}");
            result.errors.push(msg);
            has_global_error = true;
        }
    }

    if result.imported > 0 {
        log::info!(
            "[PI-SYNC] 同步完成: 导入 {} 条, 跳过 {} 条, 扫描 {} 个文件",
            result.imported,
            result.skipped,
            result.files_scanned
        );
    }

    let _ = has_global_error; // 状态已通过 errors 体现
    Ok(result)
}

/// 递归收集目录下所有 `.jsonl` 文件路径。
///
/// Pi 的会话目录包含主会话文件（`<ts>_<uuid>.jsonl`）和 subagent 会话文件
///（`<session-uuid>/<subagent-id>/run-N/session.jsonl`），两者格式一致、都需解析。
fn collect_jsonl_files(dir: &std::path::Path, out: &mut Vec<PathBuf>) {
    let entries = match fs::read_dir(dir) {
        Ok(r) => r,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_jsonl_files(&path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("jsonl") {
            out.push(path);
        }
    }
}

/// 解析 Pi assistant 消息的 usage / cost / model / timestamp。
fn parse_pi_message(
    message: &serde_json::Value,
    top: &serde_json::Value,
) -> Option<PiMessageData> {
    let usage = message.get("usage")?;

    let input_tokens = usage.get("input").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
    let output_tokens = usage.get("output").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
    let reasoning_tokens = usage
        .get("reasoning")
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as u32;
    let cache_read_tokens = usage
        .get("cacheRead")
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as u32;
    let cache_write_tokens = usage
        .get("cacheWrite")
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as u32;

    // 跳过全零 token 的消息
    if input_tokens == 0
        && output_tokens == 0
        && reasoning_tokens == 0
        && cache_read_tokens == 0
        && cache_write_tokens == 0
    {
        return None;
    }

    let cost = usage.get("cost");
    let cost_input = cost
        .and_then(|c| c.get("input"))
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
    let cost_output = cost
        .and_then(|c| c.get("output"))
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
    let cost_cache_read = cost
        .and_then(|c| c.get("cacheRead"))
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
    let cost_cache_write = cost
        .and_then(|c| c.get("cacheWrite"))
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
    let cost_total = cost
        .and_then(|c| c.get("total"))
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);

    let model_id = message
        .get("model")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();

    // 优先 message.timestamp，其次顶层 timestamp
    let timestamp_iso = message
        .get("timestamp")
        .and_then(|v| v.as_str())
        .or_else(|| top.get("timestamp").and_then(|v| v.as_str()))
        .unwrap_or("");
    let created_at = parse_pi_timestamp(timestamp_iso).unwrap_or_else(|| {
        Utc::now()
            .timestamp()
            .max(0)
    });

    // 幂等标识：优先 responseId，其次 message.id，最后用内容指纹
    let response_id = message
        .get("responseId")
        .and_then(|v| v.as_str())
        .or_else(|| message.get("id").and_then(|v| v.as_str()))
        .map(|s| s.to_string())
        .unwrap_or_else(|| {
            format!(
                "{model_id}:{created_at}:{input_tokens}:{output_tokens}:{reasoning_tokens}"
            )
        });

    Some(PiMessageData {
        input_tokens,
        output_tokens,
        reasoning_tokens,
        cache_read_tokens,
        cache_write_tokens,
        cost_input,
        cost_output,
        cost_cache_read,
        cost_cache_write,
        cost_total,
        model_id,
        created_at,
        response_id,
    })
}

/// 解析 Pi 的 ISO 时间戳（`2026-08-05T07-27-34-352Z`）为 Unix 秒。
///
/// Pi 用 `-` 分隔时分秒毫秒，非标准 RFC3339。这里把时间部分的 `-` 规范化为
/// `:` / `.` 后用 chrono 的 RFC3339 parser 解析。解析失败返回 None。
fn parse_pi_timestamp(iso: &str) -> Option<i64> {
    if iso.is_empty() {
        return None;
    }
    // 已是标准 RFC3339 的情况
    if let Ok(dt) = DateTime::parse_from_rfc3339(iso) {
        return Some(dt.timestamp());
    }
    // Pi 格式: "2026-08-05T07-27-34-352Z"
    //          0123456789012345678901234
    //          index:        11      19
    // 期望: T(10) HH(11,12) -(13) MM(14,15) -(16) SS(17,18) -(19) mmm(20-22) Z(23)
    let bytes = iso.as_bytes();
    if bytes.len() >= 20 && bytes.get(10) == Some(&b'T') {
        let mut owned: Vec<u8> = bytes.to_vec();
        let mut normalized = false;
        // 把时分秒后的 '-' 改成 ':'，毫秒前的 '-' 改成 '.'
        if owned.len() > 13 && owned[13] == b'-' {
            owned[13] = b':';
            normalized = true;
        }
        if owned.len() > 16 && owned[16] == b'-' {
            owned[16] = b':';
            normalized = true;
        }
        if owned.len() > 19 && owned[19] == b'-' {
            owned[19] = b'.';
            normalized = true;
        }
        if normalized {
            if let Ok(s) = String::from_utf8(owned) {
                if let Ok(dt) = DateTime::parse_from_rfc3339(&s) {
                    return Some(dt.timestamp());
                }
            }
        }
    }
    None
}

/// 插入单条 Pi 消息记录到 proxy_request_logs。
///
/// - `app_type = "pi"`，`provider_id = "_pi_session"`，`data_source = "pi_session"`
/// - Pi 自带明细 cost 时直接分项入库；缺失时用 cc-switch 模型定价 fallback
/// - reasoning tokens 计入 output（与 OpenCode 一致：按输出计费）
fn insert_pi_message(
    db: &Database,
    request_id: &str,
    msg: &PiMessageData,
    session_id: &str,
) -> Result<bool, AppError> {
    let conn = lock_conn!(db.conn);

    let output_with_reasoning = msg.output_tokens + msg.reasoning_tokens;

    let dedup_key = DedupKey {
        app_type: "pi",
        model: &msg.model_id,
        input_tokens: msg.input_tokens,
        output_tokens: output_with_reasoning,
        cache_read_tokens: msg.cache_read_tokens,
        cache_creation_tokens: msg.cache_write_tokens,
        created_at: msg.created_at,
    };
    if should_skip_session_insert(&conn, request_id, &dedup_key)? {
        return Ok(false);
    }

    // Pi 自带明细费用时直接用；否则用 cc-switch 模型定价计算
    let (input_cost, output_cost, cache_read_cost, cache_creation_cost, total_cost) =
        if msg.cost_total > 0.0 {
            (
                msg.cost_input.to_string(),
                msg.cost_output.to_string(),
                msg.cost_cache_read.to_string(),
                msg.cost_cache_write.to_string(),
                msg.cost_total.to_string(),
            )
        } else {
            let usage = TokenUsage {
                input_tokens: msg.input_tokens,
                output_tokens: output_with_reasoning,
                cache_read_tokens: msg.cache_read_tokens,
                cache_creation_tokens: msg.cache_write_tokens,
                model: Some(msg.model_id.clone()),
                message_id: None,
            };
            match find_model_pricing(&conn, &msg.model_id) {
                Some(pricing) => {
                    let cost = CostCalculator::calculate_for_app(
                        "pi",
                        &usage,
                        &pricing,
                        Decimal::from(1),
                    );
                    (
                        cost.input_cost.to_string(),
                        cost.output_cost.to_string(),
                        cost.cache_read_cost.to_string(),
                        cost.cache_creation_cost.to_string(),
                        cost.total_cost.to_string(),
                    )
                }
                None => (
                    "0".to_string(),
                    "0".to_string(),
                    "0".to_string(),
                    "0".to_string(),
                    "0".to_string(),
                ),
            }
        };

    let inserted_rows = conn
        .execute(
            "INSERT OR IGNORE INTO proxy_request_logs (
                request_id, provider_id, app_type, model, request_model,
                input_tokens, output_tokens, cache_read_tokens, cache_creation_tokens,
                input_cost_usd, output_cost_usd, cache_read_cost_usd, cache_creation_cost_usd, total_cost_usd,
                latency_ms, first_token_ms, status_code, error_message, session_id,
                provider_type, is_streaming, cost_multiplier, created_at, data_source
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24)",
            rusqlite::params![
                request_id,
                "_pi_session",            // provider_id
                "pi",                      // app_type
                msg.model_id,
                msg.model_id,              // request_model = model
                msg.input_tokens,
                output_with_reasoning,
                msg.cache_read_tokens,
                msg.cache_write_tokens,
                input_cost,
                output_cost,
                cache_read_cost,
                cache_creation_cost,
                total_cost,
                0i64,                      // latency_ms
                Option::<i64>::None,       // first_token_ms
                200i64,                    // status_code
                Option::<String>::None,    // error_message
                Some(session_id.to_string()),
                Some("pi_session"),        // provider_type
                1i64,                      // is_streaming
                "1.0",                     // cost_multiplier
                msg.created_at,
                "pi_session",              // data_source
            ],
        )
        .map_err(|e| AppError::Database(format!("插入 Pi 会话日志失败: {e}")))?;

    Ok(inserted_rows > 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_pi_timestamp_standard() {
        // 标准 RFC3339 应直接解析
        let ts = parse_pi_timestamp("2026-08-05T07:27:34Z");
        assert!(ts.is_some());
    }

    #[test]
    fn test_parse_pi_timestamp_dash_format() {
        // Pi 的 `-` 分隔格式
        let ts = parse_pi_timestamp("2026-08-05T07-27-34-352Z");
        assert!(ts.is_some(), "应能解析 Pi 的 dash 格式时间戳");
        // 与标准格式解析结果一致
        let std = parse_pi_timestamp("2026-08-05T07:27:34.352Z");
        assert_eq!(ts, std);
    }

    #[test]
    fn test_parse_pi_timestamp_empty() {
        assert!(parse_pi_timestamp("").is_none());
    }

    #[test]
    fn test_parse_pi_message_full() {
        let message = serde_json::json!({
            "role": "assistant",
            "model": "glm5-cdp",
            "timestamp": "2026-08-05T07-27-34-352Z",
            "responseId": "resp-123",
            "usage": {
                "input": 3088,
                "output": 155,
                "cacheRead": 0,
                "cacheWrite": 0,
                "reasoning": 86,
                "totalTokens": 3243,
                "cost": {
                    "input": 0.009264,
                    "output": 0.002325,
                    "cacheRead": 0.0,
                    "cacheWrite": 0.0,
                    "total": 0.011589
                }
            }
        });
        let top = serde_json::json!({ "id": "sess-1", "timestamp": "2026-08-05T07-27-34-352Z" });
        let data = parse_pi_message(&message, &top).unwrap();
        assert_eq!(data.input_tokens, 3088);
        assert_eq!(data.output_tokens, 155);
        assert_eq!(data.reasoning_tokens, 86);
        assert_eq!(data.cache_read_tokens, 0);
        assert_eq!(data.cache_write_tokens, 0);
        assert!((data.cost_total - 0.011589).abs() < 1e-9);
        assert_eq!(data.model_id, "glm5-cdp");
        assert_eq!(data.response_id, "resp-123");
        assert!(data.created_at > 0);
    }

    #[test]
    fn test_parse_pi_message_skips_zero_tokens() {
        let message = serde_json::json!({
            "role": "assistant",
            "model": "m",
            "usage": {
                "input": 0, "output": 0, "reasoning": 0,
                "cacheRead": 0, "cacheWrite": 0, "totalTokens": 0,
                "cost": {"input":0.0,"output":0.0,"cacheRead":0.0,"cacheWrite":0.0,"total":0.0}
            }
        });
        let top = serde_json::json!({});
        assert!(parse_pi_message(&message, &top).is_none());
    }

    #[test]
    fn test_parse_pi_message_response_id_fallback() {
        // 无 responseId 时用 message.id
        let message = serde_json::json!({
            "role": "assistant",
            "id": "msg-456",
            "model": "m",
            "timestamp": "2026-08-05T07-27-34-352Z",
            "usage": { "input": 100, "output": 10 }
        });
        let top = serde_json::json!({ "id": "sess-1" });
        let data = parse_pi_message(&message, &top).unwrap();
        assert_eq!(data.response_id, "msg-456");
    }
}
