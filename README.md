# Code Buddy

> 輕量桌面應用，透過系統匣即時監控 AI coding agent 的工作狀態。

[![Version](https://img.shields.io/badge/version-0.2.0-blue.svg)]()
[![Platform](https://img.shields.io/badge/platform-macOS-lightgrey.svg)]()
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)

## 簡介

Code Buddy 是一款基於 Tauri v2 的桌面應用，讓開發者無需切換視窗即可掌握 AI agent（Claude Code、OpenCode）的即時狀態。透過系統匣圖示變化與桌面通知，你可以知道 agent 正在思考、工作、等待輸入，或已完成任務。

### 核心功能

- **系統匣即時狀態** — 7 種狀態圖示，一眼掌握 agent 進度
- **桌面通知** — 任務完成、等待輸入、發生錯誤時主動推送
- **多 Session 監控** — 同時追蹤多個 agent session
- **Hook 驅動** — 無侵入式偵測，不依賴 agent 內部 API

## Quick Start

```bash
# 前置需求：Node.js 18+、Rust toolchain、jq
git clone https://github.com/AsiaOstrich/code-buddy.git
cd code-buddy
npm install
npm run tauri dev
```

> 完整安裝說明、環境設定與疑難排解請參閱 [DEPLOYMENT.md](DEPLOYMENT.md)。

## 技術架構

| 層級 | 技術 |
|------|------|
| 桌面框架 | Tauri v2 |
| 後端 | Rust (Axum HTTP server) |
| 前端 | React 19 + TypeScript |
| 建置工具 | Vite 6 |

> 詳細架構設計請參閱 [ARCHITECTURE.md](ARCHITECTURE.md)。

## 狀態一覽

| 狀態 | 色彩 | 說明 |
|------|------|------|
| Idle | 灰色 | 閒置 / 未連線 |
| Thinking | 紫色 | Agent 正在思考 |
| Working | 藍色 | Agent 正在執行工具 |
| Waiting Input | 黃色 | 等待使用者輸入 |
| Waiting Confirm | 橙色 | 等待使用者確認操作 |
| Completed | 綠色 | 任務完成 |
| Error | 紅色 | 發生連續錯誤 |

## 專案結構

```
code-buddy/
├── src/                  # React 前端
├── src-tauri/            # Rust 後端
│   └── src/adapters/     # Agent 轉接器
├── scripts/              # Hook 轉發與開發腳本
├── specs/                # 功能規格文件
└── .standards/           # 開發標準 (UDS Level 3)
```

## 開發指令

| 指令 | 說明 |
|------|------|
| `npm run tauri dev` | 開發模式（熱更新） |
| `npm run tauri:bg` | 背景開發模式 |
| `npm run tauri:stop` | 停止背景開發伺服器 |
| `npm run tauri build` | 建置正式版安裝檔 |

## 文件索引

| 文件 | 說明 |
|------|------|
| [ARCHITECTURE.md](ARCHITECTURE.md) | 系統架構與模組設計 |
| [DEPLOYMENT.md](DEPLOYMENT.md) | 安裝、設定與疑難排解 |
| [CHANGELOG.md](CHANGELOG.md) | 版本異動記錄 |
| [CONTRIBUTING.md](CONTRIBUTING.md) | 貢獻指南 |
| [specs/SPEC-001.md](specs/SPEC-001.md) | 完整功能規格 |

## 授權

[MIT License](LICENSE)
