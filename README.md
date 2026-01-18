# 麦当劳优惠券自动领取工具

一个用 Rust 编写的麦当劳优惠券自动领取工具，支持网页界面和终端界面两种模式。

## 功能特性

- **一键领取** - 自动领取所有可用优惠券
- **查看优惠券** - 展示已领取的优惠券列表（含图片）
- **双模式支持** - 网页模式（小白友好）和终端 TUI 模式
- **跨平台** - 支持 Windows、macOS、Linux
- **自动保存** - Token 自动保存，下次无需重复输入
- **无痕浏览** - 网页模式自动使用无痕/隐私浏览器打开

## 安装

### 从源码编译

确保已安装 [Rust](https://rustup.rs/)，然后执行：

```bash
git clone https://github.com/你的用户名/mcd-coupon.git
cd mcd-coupon
cargo build --release
```

编译后的可执行文件位于 `target/release/mcd-coupon-tui-rust`

### 直接下载

前往 [Releases](https://github.com/你的用户名/mcd-coupon/releases) 页面下载对应平台的可执行文件。

## 使用方法

### 交互式启动（推荐）

直接运行程序，会显示模式选择菜单：

```bash
./mcd-coupon-tui-rust
```

```
╔════════════════════════════════════════╗
║    麦当劳优惠券自动领取工具            ║
╠════════════════════════════════════════╣
║                                        ║
║  请选择运行模式:                       ║
║                                        ║
║  [1] 网页模式 (推荐小白用户)           ║
║      浏览器打开，界面友好              ║
║                                        ║
║  [2] 终端模式 (TUI)                    ║
║      在终端中运行，适合高级用户        ║
║                                        ║
╚════════════════════════════════════════╝

请输入选项 [1/2] (默认1):
```

- 输入 `1` 或直接回车 → 启动网页模式
- 输入 `2` → 启动终端模式

### 命令行参数

```bash
# 网页模式
./mcd-coupon-tui-rust html

# 终端模式
./mcd-coupon-tui-rust tui

# 帮助
./mcd-coupon-tui-rust --help
```

## 获取 Token

请参考麦当劳 MCP 平台官方文档：**https://open.mcd.cn/mcp/doc**

> 程序会自动添加 `Bearer ` 前缀，直接输入 Token 即可

## 配置文件

Token 会自动保存到配置文件：

| 系统 | 配置文件位置 |
|------|-------------|
| Windows | `%APPDATA%\mcd-coupon-tui-rust\config.json` |
| macOS | `~/Library/Application Support/mcd-coupon-tui-rust/config.json` |
| Linux | `~/.config/mcd-coupon-tui-rust/config.json` |

配置文件格式：
```json
{
  "token": "YOUR_TOKEN_HERE"
}
```

> 注：程序会自动添加 `Bearer ` 前缀，无需手动添加

## 技术栈

- **语言**: Rust
- **TUI 框架**: [Ratatui](https://github.com/ratatui-org/ratatui) + [Crossterm](https://github.com/crossterm-rs/crossterm)
- **Web 框架**: [Axum](https://github.com/tokio-rs/axum)
- **模板引擎**: [Handlebars](https://github.com/sunng87/handlebars-rust)
- **HTTP 客户端**: [Reqwest](https://github.com/seanmonstar/reqwest)
- **异步运行时**: [Tokio](https://tokio.rs/)

## 平台支持

| 平台 | 网页模式 | 终端模式 | 无痕浏览器 |
|------|---------|---------|-----------|
| Windows 10/11 | ✅ | ✅ | Chrome / Edge / Firefox |
| macOS | ✅ | ✅ | Chrome / Firefox |
| Linux | ✅ | ✅ | Chrome / Chromium / Firefox |

## 截图

### 网页模式

优惠券展示页面，支持图片显示：

```
┌─────────────────────────────────────┐
│  麦当劳优惠券自动领取工具           │
├─────────────────────────────────────┤
│  [一键领取所有优惠券]               │
│  [查看已领取优惠券]                 │
│  [重新设置Token]                    │
└─────────────────────────────────────┘
```

### 终端模式 (TUI)

```
┌──────────────────┬──────────────────┐
│ 操作菜单         │ 我的优惠券       │
│ ──────────       │ ──────────       │
│ [1] 领取优惠券   │ - 王牌炸鸡三拼盒 │
│ [2] 查看优惠券   │ - 薯条三重奏     │
│ [3] 重置Token    │ - 10块麦乐鸡     │
│ [q] 退出         │ ...              │
└──────────────────┴──────────────────┘
```

## 常见问题

### Q: Token 如何获取？
A: 请参考官方文档 https://open.mcd.cn/mcp/doc 获取 Token。

### Q: Token 有效期多久？
A: Token 有效期通常为几天到几周不等，失效后需要重新获取。

### Q: 端口被占用怎么办？
A: 程序会自动从 8080 开始尝试，如果被占用会自动递增端口号（最高到 9000）。

### Q: 为什么用无痕模式打开？
A: 避免浏览器缓存和 Cookie 干扰，每次都是干净的会话。

## 免责声明

本工具仅供学习和研究使用，请勿用于商业用途。使用本工具产生的任何后果由使用者自行承担。

## License

MIT License
