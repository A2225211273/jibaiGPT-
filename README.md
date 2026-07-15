# 禾序 · Codex 用量助手

“禾序”是一款面向 Windows 的 Codex 本地用量助手，开发者显示名为“沐青禾”。当前仓库处于 **UI 优先阶段**：先确认原创视觉和信息结构，再接入 Codex `app-server` 的真实用量数据。

## 当前可用内容

- 可运行的 React + TypeScript + Vite 高保真原型
- 桌面悬浮主面板
- Codex 窗口外侧贴附条演示
- 会话内结构化用量卡片演示
- 亮色与暗色主题
- 提醒设置与默认关闭的“松弛模式”演示
- ImageGen 构图概念稿

> 当前界面中的套餐、百分比、Token 和时间均为演示数据，尚未连接真实账户。

## 第一版 UI 审阅稿

当前审阅版本为 `v2`，仍保存在 `design/drafts/`，待确认后再复制到正式资源目录 `design/ui/`。

- [亮色完整稿](./design/drafts/hexu-ui-light-v2.png)（1536×1024）
- [暗色完整稿](./design/drafts/hexu-ui-dark-v2.png)（1536×1024）
- [桌面悬浮稿](./design/drafts/hexu-ui-desktop-light-v2.png)（960×760）
- [Codex 外侧贴附稿](./design/drafts/hexu-ui-attached-light-v2.png)（1240×760）
- [会话结构化卡片稿](./design/drafts/hexu-ui-conversation-light-v2.png)（1240×620）
- [ImageGen 构图概念稿](./design/drafts/hexu-ui-concept-v1.png)（1536×1024）

## 本地运行

```powershell
pnpm install
pnpm dev
```

浏览器打开 `http://127.0.0.1:4173`。也可以通过查询参数直接打开固定状态：

- `?theme=light&view=board`
- `?theme=dark&view=board`
- `?theme=light&view=desktop`
- `?theme=light&view=attached`
- `?theme=light&view=conversation`

构建验证：

```powershell
pnpm build
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

## 隐私边界（后续真实功能）

真实数据阶段只通过本机 Codex `app-server` 获取额度与用量，不读取或保存提示词、项目源码、Cookie、账户密码或认证文件。API Key 与 Bedrock 登录不会显示伪造额度。

## 计划

1. 审阅并打磨 UI
2. 建立 Tauri 2 + Rust 桌面壳
3. 接入 Codex `app-server`
4. 实现 Windows 贴附、托盘与工作提醒
5. 打包 Codex 插件与 Windows 安装包

## License

[MIT](./LICENSE)
