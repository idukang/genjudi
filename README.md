# 昆仑（Genjudi）

基于 Tauri 2 构建的跨平台（macOS / Windows / Android / iOS）AI 助手多账号隔离与快速切换工具，安装后应用名称为 **根据地**。

## 支持服务与核心特性

| 服务 | 状态 | 说明 |
| --- | --- | --- |
| **自定义 AI 网页服务** | 已启用 | 自由新增、编辑、删除任意 HTTPS AI 服务（内置 DeepSeek / ChatGPT / Claude / Gemini / Kimi / Grok 一键预设） |
| **Muse · Meta** | 已启用 | 独立会话隔离、邮箱识别、额度明细与额外额度同步、Space 预览 |
| **Hark** | 已启用 | 独立会话隔离、邮箱识别、每日/每月双上限额度同步 |
| **外部 API / CLI Agent** | 群聊可用 | 支持接入 OpenAI 兼容 API（桌面 + 手机）与本地 CLI 工具（仅桌面）参与混合群聊 |

- **原生级多账号隔离**：macOS 使用独立 UUID `WKWebsiteDataStore`，Windows 使用独立 WebView2 `UserDataFolder`，Android 使用独立 WebView `MULTI_PROFILE`。
- **多账号群聊协作（桌面全功能 · 手机混编 API 成员）**：
  - 支持创建/编辑/删除群聊，接入 Muse、Hark 与外部 OpenAI 兼容 API / 本地 CLI Agent 同群协作。
  - **Android / iOS 手机端**新增「群聊」标签页：可查看群列表、进入会话、@点名与收发消息，直连 OpenAI 兼容 API 成员参与讨论（本地 CLI 与内嵌网页成员仍需桌面端）。
  - 桌面端支持 **并行讨论**、**评审流**、**A2A 接力** 三种协作模式，支持 `@成员`、`@渠道` 与 `@全体` 点名路由；按渠道自动选号时基于剩余额度智能选择并支持失败自动冷却切换。
  - 支持 **共享上下文 & 技能（Skills）**、**成员角色设定**、**沉淀本轮结论**、**新建话题**、**站点选择器适配器自定义覆盖** 等进阶编排能力。
- **内置站点适配规则（v0.2.0）**：ChatGPT、DeepSeek、Claude、Gemini、Kimi、Grok 均内置输入框/发送按钮/回复区块/会话 URL 规则，群聊可开箱即用；仍可在「站点规则」中逐项覆盖。
- **配置备份与导出（v0.2.0）**：
  - 「设置 → 配置备份」一键导出/导入 JSON 备份（自定义服务、API 成员、群聊知识与设置），带 **SHA-256 完整性校验**，可选是否包含 API Key（空 Key 导入时保留本机值）。
  - 群聊会话支持 **导出 Markdown**（含成员、共享上下文、技能与完整对话），可复制或保存为 `.md`。
- **多端加密同步（v0.3.0）**：手机与电脑之间迁移/同步账号、代理、时区、群聊与 **Cookie 登录态**，三种方式任选：
  - **局域网直连 + 二维码扫码**：发送端生成一次性二维码（含地址、口令与端到端密钥），接收端手机直接扫码导入；3 分钟或最多 3 次拉取后自动关闭，仅监听本机内网地址，不接受公网 IPv4。
  - **加密备份文件（`.genjudi`）**：AES-256-GCM 加密整包数据，口令至少 4 位，可导出保存或复制密文，在另一台设备粘贴导入。
  - **自建 Cloudflare Worker 加密中转**：不同网络时可选，**默认关闭且不预置任何第三方地址**，需用仓库内 `relay-worker/` 部署到自己的 Cloudflare 账号（免费额度内），Worker 只暂存纯密文、无法解密。
- **保活与审批加固（v0.3.0）**：
  - 可配置「同代理仅保活当前账号」与静默保活 / 额度检测间隔（5/15/30 分钟，或设为 0 仅手动），减少多号同 IP 触发风控。
  - 自动审批只会点击 **「允许一次」** 这一类单次授权，**始终允许 / 本次会话允许 / 允许该项目** 等持久授权一律不自动点击，破坏性命令（`rm -rf`、`sudo`、`dd if=`、`curl|sh`、`git push --force`、读取 `~/.ssh` 等）出现时也会交回人工确认。
- **Cookie 导入登录与邮箱识别（全平台）**：
  - 在「新增账号」弹窗或「账号设置」弹窗中粘贴 **JSON 数组**、**Netscape `cookies.txt`** 或 **HTTP `Cookie:` 请求头** 即可导入对应账号的独立隔离容器（macOS / Windows / Android 均支持）。
  - 创建时导入会在打开页面前校验并写入 Cookie，直接进入已登录状态。
  - 自动从 Cookie、页面数据及 Meta 账户中心（`accountscenter.meta.com`）提取真实邮箱；若导入的 Cookie 未包含 `.meta.com` 凭据且无法读取邮箱，账号将标记为「已登录账号」并在下方展示 **「未获取到邮箱 · 手动改名」** 快捷按钮。
- **工具权限自动审批（桌面端）**：
  - 在左下角 **设置** 中开启「自动审批权限请求（前台与后台保活账号）」后，当前页与后台已预热保活账号会自动识别授权卡片/弹窗，**只点击「允许一次」这类单次授权**，并严格拦截删除、注销、退出登录、支付、改密等高危操作；持久授权与破坏性命令一律留给你自己确认（见下方「保活与审批加固」）。
- **登录态校验与状态异常恢复**：
  - 主动与被动检测登录态有效性，掉线时自动清理失效会话；识别错误页或受限页并标红「状态异常」与一键 **清理重登**，群聊与自动选号会自动跳过异常账号。
- **快捷切号、托盘、代理与深色模式**：
  - 支持 `⌘1~9` / `Ctrl+1~9` 及 `⌘↑`/`⌘↓`/`⌘[`/`⌘]` 快捷切号，全局呼出/隐藏快捷键，关闭最小化到系统托盘，随系统切换深色模式。
  - 支持每个账号独立 HTTP CONNECT 代理（`http://主机:端口`）及一键测速，展示认证 Cookie 剩余有效期，一键复制当前账号 Cookie，并支持客户端内 GitHub Releases `SHA-256` 校验覆盖更新。

---

## 核心功能使用说明

### 1. 多账号群聊协作（桌面 + 手机）

1. 桌面端点击左侧栏 **「群聊」→「+ 创建群聊」**；手机端点顶部 **「群聊」标签页** 进入群列表与会话。
2. 勾选要参与的多平台账号或点击 **「API/CLI」** 添加外部模型（手机端仅支持 OpenAI 兼容 API 成员）。
3. 在顶部工具栏可设置 **共享上下文 & 技能**、编辑成员角色、**沉淀本轮结论**、开启 **新话题** 或配置 **站点适配器**；点 **「导出 Markdown」** 可把当前会话存成文档。
4. 桌面端在输入框下方选择 **并行 / 评审流 / A2A 接力** 模式，输入消息并用 `@成员`、`@渠道` 或 `全体成员` 发起协作提问；手机端统一按并行方式逐一向 API 成员请求回复。

### 2. 导入 Cookie 登录（解决 Google OAuth 无法内嵌登录）

1. **两种 Cookie 导入方式的区别（Muse · Meta）**：
   - **仅导入 `muse.ai` 的 Cookie**（如复制 `Cookie:` 请求头或仅导出 `muse.ai` 单域）：可以**正常登录使用 Muse 对话、Space 与同步额度**；但由于 `muse.ai` 页面本身不返回邮箱且缺少 `.meta.com` 凭据，**无法自动识别邮箱**（Muse 设置里点「Meta 账户」也会处于未登录状态）。此时账号会显示为「已登录账号」，可在新增时填写备注名，或点击卡片下方的 **「手动改名」** 自行命名。
   - **同时导入 `muse.ai` + `meta.com` 的 Cookie（推荐）**：不仅能正常使用 Muse，应用还会**自动从 Meta 账户中心（`accountscenter.meta.com`）识别并更新真实邮箱**，点开「Meta 账户」也会保持已登录状态。
2. **多域名 Cookie 导出方法**：可将 Netscape `cookies.txt`、JSON 数组或 HTTP `Cookie:` 请求头直接粘贴；应用会自动过滤，只导入当前服务允许域名的 Cookie，无需担心混入其他网站。
3. **导入操作入口**：点击服务栏右上角 **「+ 新增」**，在 **「导入 Cookie（可选）」** 中粘贴后点击 **「创建并打开」** 即可直接进入已登录状态；也可在已有账号的 **「账号设置（编辑图标）」** 中粘贴并点击 **「导入 Cookie」** 覆盖更新。

### 3. 多端加密同步（手机 ⇄ 电脑）

入口：桌面端左侧栏底部 **「同步」**，手机端右上角同步图标。弹窗分「扫码 / 局域网」与「加密备份文件」两个标签。

1. **同一局域网（最快，推荐）**：发送端选「开启共享」（口令留空则自动生成 6 位 PIN），弹出二维码 → 手机在另一台设备上点 **「扫码直接同步」**，扫完即自动拉取并合并。机器旁也可手动输入 `genjudi-sync://…` 同步串或 `IP:端口` + 口令点「拉取并合并」。共享窗在 3 分钟或最多 3 次拉取后自动关闭。
2. **加密备份文件**：在任一设备导出 `.genjudi` 文件（设置口令，至少 4 位），拷到另一台设备后在「加密备份文件」标签选择文件或粘贴全文，输入口令点「解密并导入」。
3. **不同网络 / 开了代理时**：跨公网同步需要一个只暂存密文的中转点。勾选「使用自建 Cloudflare Worker 加密中转通道」，填入 **你自己部署的** Worker 地址（部署方法见 [`relay-worker/README.md`](relay-worker/README.md)，免费额度足够个人使用），再生成中转码扫码或粘贴即可。
4. **合并规则**：同 UUID 同服务的账号会更新备注、代理、时区并覆盖登录态；不存在的账号新增；群聊与 API 成员按 ID 合并、补齐缺失成员与消息；导入后全新账号标记为「待验证」，首次打开自动校验登录态。

**安全边界**：整包数据只用一次性 256-bit 密钥做 AES-256-GCM 端到端加密，密钥只存在于二维码/同步串/你自己填的口令中，**从不上传**；根据地不预置、不默认连接任何第三方中转服务器，局域网同步也只接受内网或本机 IPv4 地址。

---

## 下载与安装

前往 [GitHub Releases](https://github.com/idukang/genjudi/releases) 下载对应平台安装包：

| 平台 | 下载文件 | 架构 / 系统要求 |
| --- | --- | --- |
| macOS | `account-manager-macos-universal.dmg` | Apple Silicon & Intel（macOS 14.0+） |
| Windows | `account-manager-windows-x64-setup.exe` | x64（Windows 10 / 11） |
| Android | `account-manager-android-universal-release.apk` | 通用 ABI（Android 7.0+） |
| iOS | `account-manager-ios-unsigned.ipa` | 未签名 IPA（需自签名安装，iOS 17.0+） |

> `account-manager-macos-universal.app.tar.gz` 仅用于 macOS 应用内自动更新，首次安装请下载 `.dmg`。

### macOS 安装注意事项

1. **拖入应用程序目录**：打开 `account-manager-macos-universal.dmg` 后，请务必将 `根据地.app` 拖入 `/Applications`（应用程序）文件夹再启动。切勿直接在 DMG 挂载镜像中运行，否则无法写入替换，应用内自动更新将改为打开解压目录提示手动替换。
2. **提示「“根据地”已损坏，无法打开」或「无法验证开发者」**：
   - 因未购买 Apple 付费开发者证书签名与公证（Notarization），macOS Gatekeeper 会对浏览器下载的应用附加隔离属性 `com.apple.quarantine`，从而触发「已损坏」拦截提示。
   - **解决方法一（推荐，终端移除隔离属性）**：

     ```bash
     xattr -rd com.apple.quarantine /Applications/根据地.app
     ```

     若提示权限不足，可使用：

     ```bash
     sudo xattr -rd com.apple.quarantine /Applications/根据地.app
     ```

   - **解决方法二（系统设置放行）**：先点击一次「取消」，随后打开「系统设置 → 隐私与安全性」，在页面底部找到被拦截提示并点击「仍要打开」。
   - 若直接打开 `.dmg` 也提示异常，可先在终端对 `.dmg` 执行一次 `xattr -d com.apple.quarantine ~/Downloads/account-manager-macos-universal.dmg`。
3. **数据保留**：账号配置保存在 `~/Library/Application Support/com.idukang.genjudi/`，Web 登录态保存在系统 `WKWebsiteDataStore` 中。后续覆盖安装或应用内更新均不会丢失已登录账号。

### Windows 安装注意事项

1. **SmartScreen 安全拦截（「Windows 已保护你的电脑」）**：
   - 因安装包未使用付费 EV/OV 代码签名证书签名，首次运行 `account-manager-windows-x64-setup.exe` 时 Microsoft Defender SmartScreen 可能弹出拦截窗口。
   - 点击窗口中的 **「更多信息」**，再点击 **「仍要运行」** 即可正常安装。
2. **安全软件误报与校验**：
   - 若杀毒软件误报拦截，可在 Release 页面查看 `latest.json` 中 `platforms.windows.sha256` 的官方哈希，与本地安装包核对一致后放行：

     ```powershell
     Get-FileHash .\account-manager-windows-x64-setup.exe -Algorithm SHA256
     ```

3. **WebView2 运行时依赖**：
   - 依赖系统内置的 Microsoft Edge WebView2 Runtime。Windows 11 默认已自带；部分精简版或老旧 Windows 10 若首次打开空白或无法启动，请前往 [Microsoft Edge WebView2 官方下载页](https://developer.microsoft.com/microsoft-edge/webview2/) 安装「常青独立安装程序（x64）」。
4. **安装权限与数据保留**：
   - 如安装到受系统保护目录需通过 UAC 授权。账号配置与各账号独立 WebView2 数据目录均保存在 `%APPDATA%\com.idukang.genjudi\`，覆盖安装或应用内更新均不会清除已有登录状态。

### Android 安装注意事项

1. **允许安装未知来源应用**：首次安装 `.apk` 以及使用应用内更新时，系统会要求授予「允许安装未知来源应用」权限，授权后返回即可调起系统安装器覆盖更新。
2. **WebView 内核要求**：多账号隔离依赖 Android System WebView 的 `MULTI_PROFILE` 能力；如提示不支持，请在应用商店或系统更新中升级 Android System WebView（或 Chrome）。
3. **签名一致说明**：所有 Release APK 均使用固定发布密钥签名，可直接覆盖升级并完整保留多账号登录态。

---

## 应用内更新与发版

- **客户端检查更新**：桌面端点击左侧栏底部「更新」按钮，Android 端点击右上角下载图标，可检查最新版本并一键下载校验安装；启动后也会静默检查一次并展示红点提醒。
- **维护者发布新版本**：
  1. 同步更新 `package.json`、`package-lock.json`、`src-tauri/Cargo.toml`、`src-tauri/Cargo.lock` 和 `src-tauri/tauri.conf.json` 中的应用版本号。
  2. 提交并推送 `v*` 标签（GitHub Actions 将自动编译三端安装包、生成 `latest.json` 并创建 Release）：

     ```bash
     git tag v0.1.1
     git push origin v0.1.1
     ```

---

## 本地开发

环境要求：Node.js 22+、Rust 1.85+，Android 构建需额外配置 Android SDK 与 NDK 26+。

```bash
npm ci
npm run tauri -- dev
npm run tauri -- build
```

---

## 致谢与开源协议

本项目基于 [ldjx7/multi-agent-account-manager](https://github.com/ldjx7/multi-agent-account-manager)（Apache-2.0）架构学习与二次开发：多账号隔离容器、群聊编排、代理与保活体系均来自该项目的扎实实现。

### 本版从上游借鉴并移植的部分（v0.3.0）

- **多端加密同步**：`src-tauri/src/sync_bundle.rs`、`src-tauri/plugins/account-session/` 中的扫码与 Cookie 导出桥接、`src/main.ts` 的同步弹窗，以及 `relay-worker/`，均移植自上游 v0.2.0（提交 `ec85ffc`，作者 ldjx7），已按本项目品牌重命名并做了以下改写：
  - **删除了上游写死的第三方中转地址**（`https://sync.ldjx7.qzz.io`）与 `MAAM_RELAY_URL` 环境变量预置值：本项目默认无任何中转地址，必须由用户自己部署 `relay-worker/` 后填入，否则只能走局域网或加密文件。
  - 同步口令派生前缀、同步 URL scheme（`genjudi-sync://`）、备份扩展名（`.genjudi`）与加密包标识全部改为本项目标识，不再沿用上游 `maam` 命名；仍保留对旧 scheme 的解析以容错。
  - **未移植**上游 v0.2.0 的页面错误诊断上报代码（`pageDiag` / `page_error_diag.log` / 网络请求正文抓取）：它会把目标页面正文与接口返回片段写进本地日志，价值不高且扩大隐私面，本项目不需要。
- **同代理保活隔离、可配置静默间隔**：逻辑来自上游 v0.2.0 的同名功能。

### 授权审批理念参考

自动审批的「三档分流 + 只点单次授权」思路与破坏性命令特征表参考了 [Aenvo/MuseLink](https://github.com/Aenvo/MuseLink)（MIT）的 `exec_policy` 风险分级设计（`Auto` / `Session` / `Always` 三档、`destructive_delete` / `system_admin_tool` / `obfuscated_command` / `remote_force_push` 等标签）。本项目未引入其 Windows 沙箱、文件 ACL 与隧道代码（那是独立的 Windows 主机守护进程，与本应用定位不同），只把风险分级思想落成 WebView 内的审批拦截规则。

### 第三方组件

- Android 扫码使用 [journeyapps/zxing-android-embedded](https://github.com/journeyapps/zxing-android-embedded) 4.3.0（Apache-2.0）。已审阅其 AAR：仅相机与条码解析代码，无网络请求、无统计上报、无第三方地址；仅在点「扫码直接同步」时调用相机。

本项目基于 [Apache-2.0](LICENSE) 协议开源。
