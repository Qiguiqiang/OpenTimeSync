# OpenTimeSync

**English** | [**中文**](#chinese)

Cross-platform high-precision NTP time synchronization desktop app. **~6MB** binary — built with **Tauri + Rust**, no bundled Chromium.

| Platform | Download |
|----------|----------|
| Windows | `OpenTimeSync_2.0.64_x64_en-US.msi` |
| macOS | `OpenTimeSync_*.dmg` (Intel + Apple Silicon) |
| Linux | `.deb` + `.AppImage` |

→ [Latest Release](https://github.com/Qiguiqiang/OpenTimeSync/releases)

## Features

- **5 NTP servers** — Tencent / Aliyun / Apple / Google / Pool, switchable at runtime
- **Weighted averaging** — outlier-removed, precision grades S+ to D
- **Settings panel** — configurable sync interval (5–3600s), auto-sync toggle
- **Auto-update** — built-in updater checks GitHub Releases for new versions
- **14 timezones** — instant switch, persisted to localStorage
- **Cyberpunk UI** — neon glow, glass morphism, frameless window, millisecond-accurate time
- **CI/CD** — GitHub Actions auto-builds Win/Mac/Linux on tag push

## Quick Start

```bash
npm install
npm run dev        # dev mode with hot-reload
npm run build      # production build → src-tauri/target/release/bundle/
```

**Prerequisites:** Rust (stable), Node.js 22+. Linux additionally requires `libwebkit2gtk-4.1-dev`, `libgtk-3-dev`, `libayatana-appindicator3-dev`, `librsvg2-dev`, `patchelf`.

## Project Structure

```
OpenTimeSync/
├── src-tauri/           # Rust backend
│   ├── src/
│   │   ├── main.rs      # Entry point, windows_subsystem
│   │   ├── lib.rs       # Tauri commands, NTP sync loop, settings
│   │   └── ntp.rs       # NTPv3 UDP query, offset/RTT calculation
│   ├── Cargo.toml
│   └── tauri.conf.json
├── public/              # Frontend (no bundler)
│   ├── index.html
│   ├── css/style.css
│   └── js/app.js
├── .github/workflows/
└── package.json
```

## Architecture

```
NTP Server → Rust UDP socket → offset/RTT computed → stored in AppState
                                                            ↓
WebView ←─── invoke('get_ntp_status') every 2s ←─── frontend polls
```

The Rust backend queries the selected NTP server via raw UDP, computes offset and RTT using weighted averaging with outlier rejection. The frontend polls via Tauri IPC every 2 seconds and renders the corrected time with millisecond precision. All processing is local — no external server dependency.

## Precision Grades

| Grade | Offset Std Dev |
|-------|---------------|
| S+ | < 2ms |
| S | < 5ms |
| S- | < 10ms |
| A | < 30ms |
| B | < 50ms |
| C | < 100ms |
| D | ≥ 100ms |

## CI/CD

Push a tag to trigger cross-platform builds:

```bash
git tag v2.0.64
git push origin v2.0.64
```

GitHub Actions produces: Windows MSI + NSIS, macOS DMG (x86_64 + aarch64), Linux deb + AppImage.

---

<a name="chinese"></a>

# OpenTimeSync

[**English**](#features) | [**中文**](#chinese)

跨平台高精度 NTP 时间同步桌面应用。**约 6MB** 二进制体积，基于 **Tauri + Rust** 构建，不内嵌 Chromium。

| 平台 | 下载 |
|----------|------|
| Windows | `OpenTimeSync_*_x64_en-US.msi` |
| macOS | `OpenTimeSync_*.dmg` (Intel + Apple Silicon) |
| Linux | `.deb` + `.AppImage` |

→ [最新 Release](https://github.com/Qiguiqiang/OpenTimeSync/releases)

## 功能特性

- **5 个 NTP 服务器** — 腾讯云 / 阿里云 / Apple / Google / Pool，运行时可切换
- **加权平均算法** — 自动剔除异常值，精度等级 S+ ~ D
- **设置面板** — 可配置同步间隔（5–3600s），自动同步开关
- **自动更新** — 内置更新器检查 GitHub Releases 并下载安装
- **14 个时区** — 即时切换，自动保存到 localStorage
- **赛博朋克 UI** — 霓虹发光、玻璃拟态、无边框窗口、毫秒级时间显示
- **CI/CD 自动编译** — GitHub Actions 推送标签即编译全平台安装包

## 快速开始

```bash
npm install
npm run dev        # 开发模式，支持热重载
npm run build      # 生产编译 → src-tauri/target/release/bundle/
```

**环境要求:** Rust（stable）、Node.js 22+。Linux 额外需要 `libwebkit2gtk-4.1-dev`、`libgtk-3-dev`、`libayatana-appindicator3-dev`、`librsvg2-dev`、`patchelf`。

## 项目结构

```
OpenTimeSync/
├── src-tauri/           # Rust 后端
│   ├── src/
│   │   ├── main.rs      # 入口点、windows_subsystem
│   │   ├── lib.rs       # Tauri 命令、NTP 同步循环、设置
│   │   └── ntp.rs       # NTPv3 UDP 查询、偏移/RTT 计算
│   ├── Cargo.toml
│   └── tauri.conf.json
├── public/              # 前端（无打包工具）
│   ├── index.html
│   ├── css/style.css
│   └── js/app.js
├── .github/workflows/
└── package.json
```

## 架构

```
NTP 服务器 → Rust UDP socket → 计算偏移/RTT → 存入 AppState
                                                    ↓
WebView ←─── invoke('get_ntp_status') 每 2 秒 ←─── 前端轮询
```

Rust 后端通过原始 UDP socket 查询 NTP 服务器，使用加权平均 + 异常值剔除计算偏移和 RTT。前端通过 Tauri IPC 每 2 秒轮询，以毫秒精度渲染校正后的时间。所有处理在本地完成，不依赖外部服务。

## 精度等级

| 等级 | 偏移标准差 |
|-------|-----------|
| S+ | < 2ms |
| S | < 5ms |
| S- | < 10ms |
| A | < 30ms |
| B | < 50ms |
| C | < 100ms |
| D | ≥ 100ms |

## CI/CD 自动编译

推送标签触发全平台编译：

```bash
git tag v2.0.64
git push origin v2.0.64
```

GitHub Actions 自动编译：Windows MSI + NSIS、macOS DMG（x86_64 + aarch64）、Linux deb + AppImage。

---

MIT License
