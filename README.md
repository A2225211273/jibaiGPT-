# 禾序 · Codex 用量助手

“禾序”是一款面向 Windows 的 Codex 本地用量助手，开发者显示名为“沐青禾”。项目已完成原创 UI 与第二阶段桌面数据核心：Tauri 运行态会从本机 Codex `app-server` 读取真实账户用量，不用本地 Token 估算冒充官方额度。

## 当前可用内容

- Tauri 2 + Rust 的 `380×520` Windows 桌面窗口
- 通过 JSONL/JSON-RPC 连接本机 `codex app-server --listen stdio://`
- 读取真实套餐、额度窗口、重置时间、Credits 与 Token 汇总
- 每 60 秒自动刷新，并支持点击数据来源手动刷新
- ChatGPT 登录态的真实账户额度；API Key 与 Bedrock 会明确显示不支持
- 浏览器中的完整稿、桌面、贴附、会话卡片及亮/暗主题演示
- “青色序列波”品牌图标和 Windows 图标资源

> Tauri 桌面运行态使用真实数据；直接在浏览器打开的设计板仍使用演示数据。窗口吸附、托盘、工作计时、Codex 插件包与正式安装器尚未进入本阶段。

## 第一版 UI 正式稿

UI 方向已于 2026-07-15 确认，正式资源保存在 `design/ui/`；生成过程和早期对照稿仍保留在 `design/drafts/`。

- [亮色完整稿](./design/ui/hexu-ui-v1.png)（1536×1024）
- [暗色完整稿](./design/ui/hexu-ui-dark-v1.png)（1536×1024）
- [桌面悬浮稿](./design/ui/hexu-ui-desktop-v1.png)（960×760）
- [Codex 外侧贴附稿](./design/ui/hexu-ui-attached-v1.png)（1240×760）
- [会话结构化卡片稿](./design/ui/hexu-ui-conversation-v1.png)（1240×620）
- [ImageGen 构图概念稿](./design/drafts/hexu-ui-concept-v1.png)（1536×1024，仅作设计过程记录）

## 运行桌面版

开发环境需要 Windows 10 Build 19041+ 或 Windows 11 x64、Node.js/pnpm、Rust、WebView2 Runtime，以及已安装并登录的 Codex CLI。

```powershell
pnpm install
pnpm tauri:dev
```

生成不带安装器的 Debug 桌面程序：

```powershell
pnpm tauri build --debug --no-bundle
```

产物位于 `src-tauri/target/debug/hexu-codex-usage.exe`。

## 运行 UI 设计板

```powershell
pnpm dev
```

浏览器打开 `http://127.0.0.1:4173`。此模式只用于审阅 UI，可通过查询参数直接打开固定状态：

- `?theme=light&view=board`
- `?theme=dark&view=board`
- `?theme=light&view=desktop`
- `?theme=light&view=attached`
- `?theme=light&view=conversation`

构建验证：

```powershell
pnpm build
cargo test --manifest-path src-tauri/Cargo.toml
```

在本机已有 Codex 登录态时，可额外运行被默认忽略的真实数据测试：

```powershell
cargo test --manifest-path src-tauri/Cargo.toml reads_a_privacy_safe_live_snapshot -- --ignored --nocapture
```

生成固定尺寸的 UI 审阅稿（需要本机 Microsoft Edge，开发服务器需运行在 `4173` 端口）：

```powershell
pnpm capture:ui
```

## 视觉原则

- 主色：禾青 `#28B8A6`
- 画布：雾白 `#F6F8F7`
- 主文字：墨青 `#1B2725`
- 核心标识：三段青色序列波
- 不采用黑色电竞 HUD、荧光绿巨大数字、紫色套餐字样或闪电环

## Codex CLI 发现方式

禾序会依次查找：

1. `HEXU_CODEX_BIN` 指定的 Codex 可执行文件或启动器
2. 当前 `PATH` 中的 `codex.exe`、`codex.cmd` 或 `codex.bat`
3. Windows 用户 npm 全局目录中的 Codex 安装

需要覆盖自动发现结果时：

```powershell
$env:HEXU_CODEX_BIN = "C:\path\to\codex.exe"
pnpm tauri:dev
```

## 真实数据与隐私边界

桌面核心只启动本机 Codex `app-server`，完成 `initialize` / `initialized` 握手后调用 `account/read`、`account/rateLimits/read` 与 `account/usage/read`。禾序不读取或保存 `auth.json`、提示词、项目源码、Cookie、账户密码或认证令牌，也不会把邮箱传给前端。

- 只有 ChatGPT 登录态提供账户额度
- API Key 与 Bedrock 登录明确显示“不支持读取账户额度”
- app-server 没有返回的 Token 字段显示为 `—`，不会伪造数据
- Token 总数以字符串跨越 Rust/JavaScript 边界，避免大整数精度损失

## 计划

1. ✅ 审阅并打磨原创 UI
2. ✅ 建立 Tauri 2 + Rust 桌面壳并接入真实 `app-server` 数据
3. 实现 Windows 窗口贴附、托盘与工作提醒
4. 打包 Codex 插件
5. 构建 Windows 安装包、更新签名、SBOM 与哈希发布流程

## License

[MIT](./LICENSE)
