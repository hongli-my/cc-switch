# Pi Agent 支持 - 实施总结

## ✅ 已完成的工作

### 1. 后端 Rust 代码
- ✅ 在 `AppType` 枚举中添加 `Pi` 变体
- ✅ 更新 `VisibleApps` 结构以包含 Pi
- ✅ 更新 `McpApps` 和 `SkillApps` 以支持 Pi
- ✅ 创建 `pi_config.rs` 模块，包含：
  - Pi 配置文件路径管理
  - auth.json 读取/写入
  - 用量统计功能（`get_pi_usage_stats`）
  - Skill 扫描功能（`scan_pi_skills`）
- ✅ 在 `provider.rs` 中添加 Pi 的 `resolve_usage_credentials` 支持
- ✅ 更新 `lib.rs` 以包含 `pi_config` 模块

### 2. 前端 TypeScript 代码
- ✅ 在 `src/lib/api/types.ts` 中添加 `"pi"` 到 `AppId` 类型
- ✅ 更新 `App.tsx`：
  - 添加 Pi 到 `VALID_APPS`
  - 添加 Pi 到 `visibleApps` 默认值
  - 更新 `getFirstVisibleApp` 函数
- ✅ 创建 `piProviderPresets.ts` 配置文件，包含 11 个常用提供商预设
- ✅ 创建 `PiProviderForm.tsx` 表单组件

### 3. 配置和类型
- ✅ Pi 提供商预设配置（Anthropic, OpenAI, Gemini, DeepSeek, OpenRouter 等）
- ✅ 用量统计数据结构
- ✅ Skill 信息数据结构

## 🚧 待完成的工作

### 1. 后端 Tauri 命令
需要创建 Pi 专用的 Tauri 命令，类似于 `commands/hermes.rs` 或 `commands/opencode.rs`：

```rust
// commands/pi.rs
#[tauri::command]
pub fn get_pi_live_provider_ids(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    // 获取当前 Pi 配置中启用的供应商 ID 列表
}

#[tauri::command]
pub fn sync_pi_config(state: State<'_, AppState>, providers: Vec<Provider>) -> Result<(), String> {
    // 同步 CC Switch 供应商到 Pi 配置文件
}
```

### 2. 前端查询钩子
需要创建 Pi 专用的查询钩子（参考 `useOpenClaw.ts` 或 `useHermes.ts`）：

```typescript
// hooks/usePi.ts
export const usePiLiveProviderIds = () => {
  return useQuery({
    queryKey: ['piLiveProviderIds'],
    queryFn: () => providersApi.getPiLiveProviderIds(),
  });
};

export const usePiHealth = (enabled: boolean) => {
  return useQuery({
    queryKey: ['piHealth'],
    queryFn: () => piApi.checkHealth(),
    enabled,
  });
};
```

### 3. Pi 专用 UI 组件
需要创建 Pi 专用的 UI 组件：

- `src/components/pi/` 目录
  - `PiHealthBanner.tsx` - 健康检查提示
  - `PiSkillsPanel.tsx` - Skill 管理面板
  - `PiEnvPanel.tsx` - 环境变量管理
  - `PiUsagePanel.tsx` - 用量统计面板

### 4. 会话管理支持
Pi 的会话管理需要：

1. 确定 Pi 的会话存储格式（类似 Claude 的 JSONL 或 Codex 的 SQLite）
2. 实现会话解析器
3. 将会话管理集成到 SessionManager

### 5. MCP 支持
Pi 可能使用与 Claude 类似的 MCP 配置，需要：

1. 确定 Pi 的 MCP 配置文件位置
2. 实现 MCP 同步逻辑
3. 在 `mcp.rs` 中添加 Pi 支持

### 6. 用量统计脚本
创建 Pi 专用的用量统计脚本（类似于 `usage_script.rs`）：

1. 从 Pi 的 API 或会话文件提取用量数据
2. 实现 `get_usage` Tauri 命令
3. 在前端 `UsageDashboard` 中显示 Pi 用量

### 7. 设置页面集成
在设置页面中添加 Pi 的配置选项：

1. Pi 安装路径检测
2. Pi 配置文件路径自定义
3. Pi 专用设置（如默认模型、温度等）

### 8. 国际化
添加 Pi 相关的翻译字符串到 i18n 文件：

```json
{
  "pi": {
    "name": "Pi Agent",
    "addProvider": "添加 Pi 供应商",
    "editProvider": "编辑 Pi 供应商",
    "providerFormDescription": "配置 Pi Agent 的 API 供应商",
    "preset": "预设",
    "selectPreset": "选择预设...",
    "providerType": "提供商类型",
    "apiKeyPlaceholder": "输入 API Key",
    "notesPlaceholder": "可选：添加备注"
  }
}
```

### 9. 测试
需要添加：

1. 后端单元测试（`pi_config.rs` 已有部分测试）
2. 前端组件测试
3. 集成测试（添加/编辑/删除 Pi 供应商）
4. E2E 测试（完整工作流程）

## 📝 下一步建议

### 优先级 1（核心功能）
1. 实现 `commands/pi.rs` - Pi 专用 Tauri 命令
2. 创建 `hooks/usePi.ts` - Pi 查询钩子
3. 更新 `App.tsx` 以添加 Pi 的视图和路由

### 优先级 2（功能完善）
4. 实现 Pi Skill 管理
5. 实现 Pi 用量统计
6. 添加 Pi MCP 支持

### 优先级 3（用户体验）
7. 添加 Pi 健康检查和错误提示
8. 完善 Pi 设置页面
9. 添加国际化支持

## 🔍 参考文件

已实现的关键文件：
- `src-tauri/src/app_config.rs` - AppType 枚举和 VisibleApps
- `src-tauri/src/pi_config.rs` - Pi 配置管理
- `src-tauri/src/provider.rs` - 供应商凭据解析
- `src/lib/api/types.ts` - 前端类型定义
- `src/config/piProviderPresets.ts` - 提供商预设
- `src/components/providers/forms/PiProviderForm.tsx` - 提供商表单

待参考的实现：
- `src-tauri/src/hermes_config.rs` - Hermes 配置（类似 Pi）
- `src-tauri/src/commands/hermes.rs` - Hermes 命令
- `src/hooks/useHermes.ts` - Hermes 钩子
- `src/components/hermes/` - Hermes 组件

## 💡 提示

Pi Agent 的配置结构与 OpenClaw 和 Hermes 类似（additive mode），可以参考它们的实现：

1. **配置结构**：Pi 使用 `~/.pi/agent/auth.json`，类似于 Hermes 的 `config.yaml`
2. **供应商模式**：Pi 应该使用 additive mode（所有供应商都写入配置）
3. **API 兼容**：Pi 支持多种 OpenAI 兼容的 API，需要灵活的提供商配置

## ✨ 总结

目前已完成了 Pi Agent 支持的基础架构，包括类型定义、配置管理、预设配置和基本的 UI 组件。接下来需要实现核心的 Tauri 命令和查询钩子，以及 Pi 专用的 UI 面板（Skill、用量统计、MCP 等）。
