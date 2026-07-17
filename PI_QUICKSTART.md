# Pi Agent 快速入门指南

## 概述

CC Switch 现在已经支持 Pi Agent！Pi 是一个轻量级的终端编程助手，支持多种 API 提供商。

## 功能特性

✅ **已支持的功能**：
- 添加/编辑/删除 Pi 供应商
- 支持 11 种常用提供商预设（Anthropic, OpenAI, Gemini, DeepSeek 等）
- 供应商配置管理（API Key, Base URL）
- 用量统计（从会话文件解析）
- Skill 扫描和管理

🚧 **正在开发的功能**：
- Pi 专用 UI 面板
- 实时健康检查
- MCP 集成
- 会话管理

## 如何使用

### 1. 启用 Pi 应用

1. 打开 CC Switch
2. 点击设置（齿轮图标）
3. 在"应用可见性"部分，启用 **Pi Agent**
4. 保存设置

### 2. 添加 Pi 供应商

#### 方法一：使用预设（推荐）

1. 在主界面，切换到 **Pi** 应用
2. 点击"添加供应商"按钮（+）
3. 在弹出的表单中，选择一个预设（如 "Anthropic (Claude)"）
4. 输入 API Key
5. 点击"添加"

#### 方法二：自定义提供商

1. 选择 "Custom (OpenAI Compatible)" 预设
2. 输入自定义名称
3. 填写 API Key 和 Base URL
4. 点击"添加"

### 3. 配置预设列表

目前支持的预设：

| 预设 | 提供商类型 | 说明 |
|------|-----------|------|
| Anthropic (Claude) | anthropic | Claude 模型 |
| OpenAI | openai | GPT 模型 |
| Google Gemini | google | Gemini 模型 |
| DeepSeek | deepseek | DeepSeek 模型 |
| OpenRouter | openrouter | 聚合多个模型 |
| Groq | groq | 高速推理 |
| Mistral AI | mistral | Mistral 模型 |
| xAI (Grok) | xai | Grok 模型 |
| Together AI | together | Together AI |
| Fireworks AI | fireworks | Fireworks AI |
| Custom | custom | 自定义 OpenAI 兼容 API |

### 4. 用量统计

Pi 的用量统计会自动从会话文件中解析：

1. CC Switch 会扫描 `~/.pi/agent/sessions/` 目录
2. 读取每个会话的 `metadata.json` 文件
3. 提取 `total_tokens` 和 `last_active` 信息
4. 在"用量统计"页面显示

### 5. Skill 管理

Pi 的 Skill 扫描功能：

1. CC Switch 会扫描 `~/.pi/agent/skills/` 目录
2. 读取每个 Skill 的 `SKILL.md` 文件
3. 提取 Skill 名称和描述
4. 在"Skills"页面显示（即将支持）

## 配置文件位置

Pi Agent 的配置文件位于：

- **配置目录**: `~/.pi/agent/`
- **认证文件**: `~/.pi/agent/auth.json`
- **模型配置**: `~/.pi/agent/models.json`
- **会话目录**: `~/.pi/agent/sessions/`
- **Skill 目录**: `~/.pi/agent/skills/`

## 与 Pi 官方工具的集成

CC Switch 会读取和写入 Pi 的配置文件：

1. **读取**: CC Switch 可以导入 Pi `auth.json` 中已配置的供应商
2. **写入**: 在 CC Switch 中添加的供应商可以选择同步到 Pi `auth.json`

## 故障排除

### Pi 应用不显示

**问题**: 在应用切换器中看不到 Pi 选项

**解决**:
1. 检查设置中的"应用可见性"，确保 Pi Agent 已启用
2. 重启 CC Switch

### 无法添加供应商

**问题**: 点击"添加供应商"没有反应

**解决**:
1. 检查浏览器控制台是否有错误
2. 确保已正确配置 Rust 后端
3. 重新编译后端：`cargo build`

### 用量统计不准确

**问题**: 用量统计显示为 0 或不正确

**解决**:
1. 确保 Pi 会话目录存在：`~/.pi/agent/sessions/`
2. 检查会话文件中是否有 `metadata.json`
3. 手动触发用量统计刷新

## 开发者信息

### 项目结构

```
cc-switch/
├── src-tauri/src/
│   ├── pi_config.rs          # Pi 配置管理（新增）
│   ├── app_config.rs         # AppType 枚举（已更新）
│   ├── provider.rs           # 供应商凭据解析（已更新）
│   └── lib.rs                # 模块导出（已更新）
└── src/
    ├── config/
    │   └── piProviderPresets.ts   # Pi 预设配置（新增）
    ├── components/
    │   └── providers/forms/
    │       └── PiProviderForm.tsx  # Pi 表单组件（新增）
    └── lib/
        └── api/
            └── types.ts          # AppId 类型（已更新）
```

### 编译和测试

```bash
# 编译 Rust 后端
cd cc-switch
cargo build

# 运行前端开发服务器
pnpm dev:renderer

# 运行完整应用
pnpm tauri dev
```

### 下一步开发

查看 `PI_IMPLEMENTATION.md` 了解待完成的功能和开发路线图。

## 相关链接

- [Pi Agent 官网](https://pi.dev)
- [Pi Agent GitHub](https://github.com/earendil-works/pi-mono)
- [Pi Agent 文档](https://github.com/earendil-works/pi-mono/tree/main/packages/ai/docs)
- [CC Switch GitHub](https://github.com/farion1231/cc-switch)

## 反馈和支持

如果你遇到问题或有建议，请：

1. 在 [GitHub Issues](https://github.com/farion1231/cc-switch/issues) 提交问题
2. 加入 [Discord 社区](https://discord.com/invite/3cU7Bz4UPx)
3. 查看 [PI_IMPLEMENTATION.md](./PI_IMPLEMENTATION.md) 了解开发进度
