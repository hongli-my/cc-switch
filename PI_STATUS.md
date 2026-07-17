# Pi Agent 支持 - 当前状态报告

## 📊 完成进度

### ✅ 已完成（约 60%）

#### 1. 核心类型定义
- ✅ `AppType` 枚举添加 `Pi` 变体
- ✅ `AppId` 前端类型添加 `"pi"`
- ✅ `VisibleApps` 添加 `pi` 字段
- ✅ `McpApps` 添加 `pi` 字段和方法
- ✅ `SkillApps` 添加 `pi` 字段和方法

#### 2. 配置管理
- ✅ 创建 `pi_config.rs` 模块
  - ✅ 配置文件路径管理
  - ✅ auth.json 读取/写入
  - ✅ 用量统计功能
  - ✅ Skill 扫描功能

#### 3. 供应商凭据解析
- ✅ `provider.rs` 中 `resolve_usage_credentials` 添加 Pi 支持

#### 4. 前端组件
- ✅ `piProviderPresets.ts` - 11 个提供商预设
- ✅ `PiProviderForm.tsx` - 供应商表单组件
- ✅ `App.tsx` 更新（VALID_APPS, visibleApps）

#### 5. 设置
- ✅ `settings.rs` 中 `VisibleApps` 添加 `pi`
- ✅ `get_current_provider` 函数添加 Pi 分支
- ✅ `set_current_provider` 函数添加 Pi 分支

### 🚧 待完成（约 40%）

#### 1. 修复编译错误（高优先级）
当前有 **41 个编译错误**，主要是因为：
- 很多 `match` 语句没有处理 `AppType::Pi`
- 需要为所有使用 `AppType` 的 match 添加 Pi 分支

**主要文件需要修复**：
- `src/app_config.rs` - 还有多个 match 需要修复（提示词、MCP 配置等）
- `src/commands/config.rs` - 配置命令
- `src/services/skill.rs` - Skill 服务（已部分修复）
- `src/services/config.rs` - 配置服务
- `src/provider.rs` - 供应商服务
- `src/lib.rs` - 主入口
- `src/prompt_files.rs` - 提示词文件
- `src/deeplink/provider.rs` - Deep link
- `src/proxy/providers/mod.rs` - 代理提供商

#### 2. Pi 专用功能（中优先级）
- ❌ Pi 的 Tauri 命令（`commands/pi.rs`）
- ❌ Pi 查询钩子（`hooks/usePi.ts`）
- ❌ Pi 健康检查
- ❌ Pi 配置同步
- ❌ Pi 会话管理
- ❌ Pi MCP 集成

#### 3. UI 组件（低优先级）
- ❌ Pi 专用面板（Skill、用量、Env）
- ❌ Pi 在 AppSwitcher 中的图标
- ❌ Pi 设置页面

## 🔧 下一步行动建议

### 立即执行（修复编译）

**方案 1：手动修复所有 match（推荐但耗时）**
```bash
# 查找所有需要处理的位置
cd src-tauri/src
grep -rn "AppType::Hermes" --include="*.rs" | grep -v "test"

# 在每个 match 中的 AppType::Hermes 后添加 AppType::Pi
```

**方案 2：使用通配符临时修复（快速但不完整）**
在一些不重要的 match 中使用 `_ =>` 通配符，但这样会失去编译时检查。

**方案 3：为 AppType 添加默认实现**
为 AppType 实现一些辅助方法，减少 match 的使用。例如：
```rust
impl AppType {
    pub fn supports_mcp(&self) -> bool {
        !matches!(self, AppType::OpenClaw | AppType::ClaudeDesktop)
    }
    
    pub fn default_skills_dir(&self) -> PathBuf {
        // 返回默认路径，避免在每个 match 中重复
    }
}
```

### 中期目标（功能完善）

1. 创建 `commands/pi.rs` - Pi 专用命令
2. 创建 `hooks/usePi.ts` - Pi 查询钩子
3. 实现 Pi 配置同步（CC Switch ↔ Pi auth.json）
4. 实现 Pi 用量统计
5. 实现 Pi Skill 管理

### 长期目标（用户体验）

1. Pi 专用 UI 面板
2. Pi 会话管理集成
3. Pi MCP 支持
4. 完整的国际化支持

## 📝 技术债务

1. **编译错误**：需要系统性地修复所有 match 语句
2. **测试覆盖**：目前几乎没有测试
3. **文档完善**：需要更新用户文档和开发者文档
4. **错误处理**：Pi 相关函数的错误处理需要完善

## 💡 临时解决方案

如果你现在想测试 Pi 支持，可以：

1. **注释掉 Pi**：暂时从 `AppType` 中移除 `Pi`，先让项目编译通过
2. **使用 `_` 通配符**：在 match 语句中使用 `_ =>` 临时处理 Pi
3. **聚焦核心功能**：先修复关键的 match（如供应商管理），其他的用 `todo!()` 占位

## 🎯 推荐工作流程

1. **今天**：修复编译错误（预计 2-3 小时）
   - 使用脚本或手动修复所有 match
   - 确保每个 match 都有 `AppType::Pi` 分支（即使是 `todo!()`）

2. **明天**：实现 Pi 核心功能（预计 4-6 小时）
   - 创建 `commands/pi.rs`
   - 实现配置同步
   - 测试供应商添加/切换

3. **后天**：完善 UI 和功能（预计 6-8 小时）
   - 创建 Pi 专用组件
   - 实现用量统计
   - 集成 Skill 管理

## 📞 需要帮助？

如果遇到问题，可以：
1. 查看 `PI_IMPLEMENTATION.md` 了解详细计划
2. 查看 `PI_QUICKSTART.md` 了解使用方法
3. 参考 `hermes_config.rs` 和 `opencode_config.rs` 的实现
4. 在 GitHub Issues 中提问

---

**当前状态**：基础架构已完成，但需要修复编译错误才能继续。建议优先解决编译问题。
