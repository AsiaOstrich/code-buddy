# Changelog

All notable changes to this project will be documented in this file.

Format based on [Keep a Changelog](https://keepachangelog.com/).

## [0.2.0] - 2026-02-26

### Added
- Hook 驅動的 agent 狀態偵測系統
- Claude Code Adapter：支援 8 種 hook 事件（SessionStart、UserPromptSubmit、PostToolUse、PostToolUseFailure、Stop、Notification、SessionEnd）
- Axum HTTP server 監聽 `localhost:19199`
- Agent 狀態機：7 種狀態（Idle、Thinking、Working、WaitingInput、WaitingConfirm、Completed、Error）
- 多 Session 追蹤與狀態聚合策略（pinned > focus > aggregate）
- 桌面通知推送（含 30 秒防重複機制）
- PostToolUseFailure 防抖機制（連續 3 次才轉 Error）
- `claude-hook.sh` 事件轉發腳本
- Dev Panel：即時 Session 列表 + 狀態測試按鈕
- AgentAdapter trait 定義（為未來 OpenCode 等 adapter 預留擴展介面）

### Changed
- Dev Panel 改為 debug 模式自動開啟，release 模式隱藏

### Fixed
- 改善 Dev Panel 存取方式
- 新增背景開發模式腳本（`tauri:bg` / `tauri:stop`）

## [0.1.0] - 2026-02-26

### Added
- Tauri v2 桌面應用基礎架構
- 系統匣圖示與右鍵菜單（顯示面板 / 關於 / 退出）
- React 19 + TypeScript + Vite 6 前端框架
- macOS Accessory 模式（release 時背景常駐）
- 基礎專案結構與建置流程

## [0.0.0] - 2026-02-26

### Added
- 專案初始化
- UDS Level 3 開發標準配置
- CONTRIBUTING.md 貢獻指南
- SPEC-001 完整功能規格文件
