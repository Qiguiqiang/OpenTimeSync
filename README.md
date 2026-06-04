# OpenTimeSync

[English](#english) | [中文](#chinese)

---

<a name="english"></a>

# OpenTimeSync

Cross-platform high-precision time synchronization desktop app via NTP-weighted clock.

Built with Electron + Node.js — runs on **Windows, macOS, Linux**.

## Features

- **NTP Server Selection** — Choose from 5 NTP servers (Tencent/Aliyun/Apple/Google/Pool). Default: ntp.tencent.com
- **High-Precision Sync** — Weighted averaging with outlier filtering, precision grading S+ to D
- **NTP Offset Display** — Shows real deviation between local clock and NTP time
- **Network Latency** — Per-server RTT displayed in real-time
- **14 Timezones** — Quick switch with cyberpunk dropdown panel
- **Cyberpunk UI** — Dark theme with neon glow, glass morphism, flowing light animation
- **Responsive Layout** — Scales smoothly from 600px to 4K
- **Live UTC Display** — Continuous millisecond-accurate time rendering

## Download

Get the latest build from [GitHub Releases](https://github.com/Qiguiqiang/timeSyncWord/releases):

| Platform | File | Type |
|----------|------|------|
| Windows | `OpenTimeSync-*-Setup.exe` | Installer |
| Windows | `OpenTimeSync-*-Portable.exe` | Portable (no install) |
| Windows | `OpenTimeSync-*-win32-x64.zip` | ZIP archive |
| macOS | `OpenTimeSync-*.dmg` | DMG installer |
| Linux | `OpenTimeSync-*.AppImage` | Portable (no install) |
| Linux | `opentimesync_*_amd64.deb` | Deb package |

## Quick Start (Development)

```bash
# Install dependencies
npm install

# Start server-only mode (browser)
npm start
# Open http://localhost:13013

# Start Electron desktop app
npm run electron
```

## Build

```bash
# Package for current platform
npm run build:win     # Windows
npm run build:mac     # macOS
npm run build:linux   # Linux
npm run build:all     # All platforms
```

## Project Structure

```
OpenTimeSync/
├── server/              # Backend (Express + WebSocket + NTP)
│   ├── index.js         # HTTP server + WebSocket setup
│   ├── config.js        # NTP servers, ports, sync params
│   ├── time-service.js  # NTP weighted sync engine
│   └── signaling.js     # WebSocket broadcast + NTP selector
├── public/              # Frontend UI
│   ├── index.html       # Main page
│   ├── css/style.css    # Cyberpunk theme
│   └── js/app.js        # Time sync + NTP/TZ selectors
├── electron/            # Desktop shell
│   ├── main.js          # Electron main process
│   └── preload.js       # Security preload script
├── build/               # App icons (SVG/PNG/ICO)
├── .github/workflows/   # CI/CD: auto-build Win/Mac/Linux
└── package.json         # Dependencies + build config
```

## Precision Grades

| Grade | Offset Std Dev | Description |
|-------|---------------|-------------|
| S+ | < 2ms | Extremely stable |
| S | < 5ms | Very stable |
| S- | < 10ms | Stable |
| A | < 30ms | Good |
| B | < 50ms | Fair |
| C | < 100ms | Poor |
| D | >= 100ms | Unstable |

## CI/CD

Push a tag to trigger automatic cross-platform builds:

```bash
git tag v1.0.0
git push origin v1.0.0
```

GitHub Actions compiles Windows (NSIS + Portable + ZIP), macOS (DMG), and Linux (AppImage + deb) automatically.

## Architecture

```
NTP Server (selected) → Local Node.js → WebSocket → Electron Window
                              ↓
                     NTP offset + RTT displayed in UI
```

All processing is local — no external server dependency.

## Configuration

Edit `server/config.js`:
- `port`: HTTP port (default 13013)
- `ntpServers`: Available NTP server list
- `defaultServer`: Default selected NTP server
- `sync.samplesPerServer`: Samples per NTP server
- `sync.resyncInterval`: NTP resync interval (ms)

---

<a name="chinese"></a>

# OpenTimeSync

基于 NTP 加权时钟的高精度时间同步跨平台桌面应用。

基于 Electron + Node.js 构建，支持 **Windows、macOS、Linux**。

## 功能特性

- **NTP 服务器选择** — 5 个 NTP 服务器可选（腾讯云/阿里云/Apple/Google/Pool），默认 ntp.tencent.com
- **高精度同步** — 加权平均 + 异常值过滤，精度等级 S+ 到 D
- **NTP 偏差显示** — 实时展示本机时钟与 NTP 标准时间的偏差
- **网络延迟** — 每个服务器的实时 RTT 延迟显示
- **14 个时区** — 赛博朋克下拉面板快速切换
- **赛博朋克 UI** — 暗色主题、霓虹发光、玻璃拟态、流水光效
- **响应式布局** — 从 600px 到 4K 流畅缩放
- **实时 UTC** — 毫秒级精度持续时间渲染

## 下载

最新编译版本在 [GitHub Releases](https://github.com/Qiguiqiang/timeSyncWord/releases) 页面：

| 平台 | 文件 | 类型 |
|----------|------|------|
| Windows | `OpenTimeSync-*-Setup.exe` | 安装包 |
| Windows | `OpenTimeSync-*-Portable.exe` | 便携版（无需安装） |
| Windows | `OpenTimeSync-*-win32-x64.zip` | ZIP 压缩包 |
| macOS | `OpenTimeSync-*.dmg` | DMG 安装包 |
| Linux | `OpenTimeSync-*.AppImage` | 便携版（无需安装） |
| Linux | `opentimesync_*_amd64.deb` | Deb 包 |

## 快速开始（开发模式）

```bash
# 安装依赖
npm install

# 仅启动服务端（浏览器访问）
npm start
# 打开 http://localhost:13013

# 启动 Electron 桌面应用
npm run electron
```

## 本地打包

```bash
# 打包当前平台
npm run build:win     # Windows
npm run build:mac     # macOS
npm run build:linux   # Linux
npm run build:all     # 全平台
```

## 项目结构

```
OpenTimeSync/
├── server/              # 后端（Express + WebSocket + NTP）
│   ├── index.js         # HTTP 服务 + WebSocket
│   ├── config.js        # NTP 服务器、端口、同步参数
│   ├── time-service.js  # NTP 加权同步引擎
│   └── signaling.js     # WebSocket 广播 + NTP 选择器
├── public/              # 前端 UI
│   ├── index.html       # 主页面
│   ├── css/style.css    # 赛博朋克主题
│   └── js/app.js        # 时间同步 + NTP/时区选择器
├── electron/            # 桌面壳
│   ├── main.js          # Electron 主进程
│   └── preload.js       # 安全预加载脚本
├── build/               # 应用图标（SVG/PNG/ICO）
├── .github/workflows/   # CI/CD: 自动编译 Win/Mac/Linux
└── package.json         # 依赖 + 构建配置
```

## 精度等级

| 等级 | 偏移标准差 | 说明 |
|-------|---------------|-------------|
| S+ | < 2ms | 极其稳定 |
| S | < 5ms | 非常稳定 |
| S- | < 10ms | 稳定 |
| A | < 30ms | 良好 |
| B | < 50ms | 一般 |
| C | < 100ms | 较差 |
| D | >= 100ms | 不稳定 |

## CI/CD 自动编译

推送标签自动触发全平台编译：

```bash
git tag v1.0.0
git push origin v1.0.0
```

GitHub Actions 自动编译 Windows（NSIS + 便携版 + ZIP）、macOS（DMG）和 Linux（AppImage + deb）。

## 架构

```
选择的 NTP 服务器 → 本地 Node.js → WebSocket → Electron 窗口
                              ↓
                     显示 NTP 偏差 + RTT 延迟
```

所有处理在本地完成，无需外部服务器。

## 配置

编辑 `server/config.js`：
- `port`：HTTP 端口（默认 13013）
- `ntpServers`：可用 NTP 服务器列表
- `defaultServer`：默认 NTP 服务器
- `sync.samplesPerServer`：每个 NTP 服务器的采样数
- `sync.resyncInterval`：NTP 重新同步间隔（毫秒）

## 许可证

MIT