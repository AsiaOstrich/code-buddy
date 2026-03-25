# SPEC-002 Code Buddy — 跨平台安裝發行

> **Status**: Approved
> **Author**: AlbertHsu
> **Created**: 2026-03-25
> **Last Updated**: 2026-03-25
> **Depends on**: SPEC-001

---

## Overview

建立 Code Buddy 的跨平台自動化發行管線，讓開發者可透過 `curl` 一鍵安裝或 `brew install` 安裝，無需設定 Rust 建置環境。

## Motivation

- 目前安裝 Code Buddy 需要 Rust + Node.js 環境，門檻過高
- 目標使用者是開發者，但不代表每個人都想裝 Rust toolchain
- 需要跨平台（macOS / Linux / Windows）支援
- 希望一行指令即可完成安裝

## Requirements

### Requirement: CI/CD 跨平台自動建置

系統 SHALL 在 git tag 推送時自動觸發跨平台建置並發布到 GitHub Releases。

#### Scenario: Tag 觸發自動建置
- **GIVEN** 維護者推送 `v*` 格式的 git tag（如 `v0.3.0`）
- **WHEN** GitHub Actions workflow 被觸發
- **THEN** `tauri build` 產出以下 6 個平台的安裝檔：
  - macOS arm64 (Apple Silicon)
  - macOS x86_64 (Intel)
  - Linux x86_64
  - Linux arm64
  - Windows x86_64
  - Windows arm64

#### Scenario: 自動發布到 GitHub Releases
- **GIVEN** 所有平台建置成功
- **WHEN** artifacts 準備完成
- **THEN** 自動上傳至 GitHub Releases，包含：
  - 各平台的安裝檔（`.dmg`, `.AppImage`, `.msi` 或解壓縮即可用的 binary）
  - 每個檔案的 SHA256 checksum
  - 平台識別資訊在檔名中（如 `code-buddy-v0.3.0-darwin-arm64.tar.gz`）

#### Scenario: 建置失敗不發布
- **GIVEN** 任一平台建置失敗
- **WHEN** CI 偵測到錯誤
- **THEN** 不建立 GitHub Release，維護者收到失敗通知

### Requirement: curl 一鍵安裝腳本

系統 SHALL 提供跨平台安裝腳本，一行指令完成安裝。

#### Scenario: macOS 安裝
- **GIVEN** 使用者在 macOS（arm64 或 x86_64）上
- **WHEN** 執行 `curl -fsSL https://raw.githubusercontent.com/AsiaOstrich/code-buddy/main/install.sh | sh`
- **THEN** 安裝腳本偵測 OS 和 CPU 架構，從 GitHub Releases 下載對應 binary，安裝至 `~/.local/bin/code-buddy`，並輸出安裝成功訊息

#### Scenario: Linux 安裝
- **GIVEN** 使用者在 Linux（x86_64 或 arm64）上
- **WHEN** 執行相同的 curl 命令
- **THEN** 行為與 macOS 相同，下載 Linux 對應 binary

#### Scenario: Windows 安裝
- **GIVEN** 使用者在 Windows 上使用 PowerShell
- **WHEN** 執行 `irm https://raw.githubusercontent.com/AsiaOstrich/code-buddy/main/install.ps1 | iex`
- **THEN** 下載 Windows binary，安裝至 `%LOCALAPPDATA%\code-buddy\`，並加入 PATH

#### Scenario: 安裝後驗證
- **GIVEN** 安裝完成
- **WHEN** 使用者執行 `code-buddy --version`
- **THEN** 輸出當前版本號（如 `code-buddy 0.3.0`）

#### Scenario: 安裝後設定 Claude Code Hook
- **GIVEN** 安裝完成
- **WHEN** 使用者執行 `code-buddy setup`
- **THEN** 自動建立 Claude Code hook 設定，指向已安裝的 Code Buddy HTTP endpoint

#### Scenario: 已安裝時的升級
- **GIVEN** Code Buddy 已安裝舊版本
- **WHEN** 再次執行安裝腳本
- **THEN** 偵測到已安裝，下載新版本覆蓋舊版本，輸出升級成功訊息

#### Scenario: 無網路或下載失敗
- **GIVEN** GitHub Releases 無法連線或下載中斷
- **WHEN** 安裝腳本嘗試下載
- **THEN** 輸出明確錯誤訊息並以非零狀態碼退出，不留下不完整的安裝

### Requirement: Homebrew tap 安裝

系統 SHALL 提供 Homebrew Formula，讓 macOS/Linux 使用者可透過 Homebrew 安裝。

#### Scenario: Homebrew 首次安裝
- **GIVEN** 使用者已安裝 Homebrew
- **WHEN** 執行 `brew install asiaostrich/tap/code-buddy`
- **THEN** Homebrew 自動 tap `AsiaOstrich/homebrew-tap` repo，下載預編譯 binary，安裝至 Homebrew 管理的路徑

#### Scenario: Homebrew 升級
- **GIVEN** Code Buddy 已透過 Homebrew 安裝
- **WHEN** 新版本發布且 Formula 已更新，使用者執行 `brew upgrade code-buddy`
- **THEN** 下載新版本 binary 並替換舊版本

#### Scenario: Formula 自動更新
- **GIVEN** GitHub Releases 發布新版本
- **WHEN** CI workflow 偵測到新 release
- **THEN** 自動更新 `AsiaOstrich/homebrew-tap` repo 中的 Formula（版本號 + SHA256）

## Acceptance Criteria

| AC | 說明 | 歸屬 |
|----|------|------|
| AC-1 | CI 可建置 macOS arm64/x86_64 二進位檔 | CI/CD |
| AC-2 | CI 可建置 Linux x86_64/arm64 二進位檔 | CI/CD |
| AC-3 | CI 可建置 Windows x86_64/arm64 二進位檔 | CI/CD |
| AC-4 | Git tag 觸發自動發布到 GitHub Releases（含 checksum） | CI/CD |
| AC-5 | `curl ... \| sh` 在 macOS 上正確安裝 | curl 腳本 |
| AC-6 | `curl ... \| sh` 在 Linux 上正確安裝 | curl 腳本 |
| AC-7 | PowerShell 腳本在 Windows 上正確安裝 | curl 腳本 |
| AC-8 | 安裝後 `code-buddy --version` 輸出版本號 | curl 腳本 |
| AC-9 | `code-buddy setup` 自動設定 Claude Code hook | curl 腳本 |
| AC-10 | `brew install asiaostrich/tap/code-buddy` 可安裝 | Homebrew |
| AC-11 | `brew upgrade code-buddy` 可升級 | Homebrew |
| AC-12 | 新版本發布時 Formula 自動更新 | Homebrew |

## Technical Design

### Architecture Overview

```
┌──────────────────────────────────────────────────┐
│                Git Push Tag v*                    │
└──────────────────┬───────────────────────────────┘
                   │
                   ▼
┌──────────────────────────────────────────────────┐
│           GitHub Actions Workflow                 │
│                                                  │
│  ┌────────────┐ ┌────────────┐ ┌──────────────┐ │
│  │ macOS      │ │ Linux      │ │ Windows      │ │
│  │ arm64      │ │ x86_64     │ │ x86_64       │ │
│  │ x86_64     │ │ arm64      │ │ arm64        │ │
│  └─────┬──────┘ └─────┬──────┘ └──────┬───────┘ │
│        │              │               │          │
│        └──────────────┼───────────────┘          │
│                       ▼                          │
│              GitHub Releases                     │
│  ┌────────────────────────────────────────────┐  │
│  │ code-buddy-v0.3.0-darwin-arm64.tar.gz     │  │
│  │ code-buddy-v0.3.0-darwin-x86_64.tar.gz    │  │
│  │ code-buddy-v0.3.0-linux-x86_64.tar.gz     │  │
│  │ code-buddy-v0.3.0-linux-arm64.tar.gz      │  │
│  │ code-buddy-v0.3.0-windows-x86_64.zip      │  │
│  │ code-buddy-v0.3.0-windows-arm64.zip        │  │
│  │ checksums.sha256                           │  │
│  └────────────────────────────────────────────┘  │
└──────────────────┬───────────────┬───────────────┘
                   │               │
          ┌────────┘               └────────┐
          ▼                                 ▼
┌──────────────────┐            ┌───────────────────┐
│   install.sh     │            │  homebrew-tap      │
│   install.ps1    │            │  Formula 自動更新   │
│                  │            │                    │
│ curl ... | sh    │            │ brew install ...   │
└──────────────────┘            └───────────────────┘
```

### CI Workflow: `.github/workflows/release.yml`

使用 `tauri-apps/tauri-action` 官方 Action：
- Trigger: `on: push: tags: ['v*']`
- Matrix: macOS (arm64, x86_64), Linux (x86_64, arm64), Windows (x86_64, arm64)
- Output: `tauri build` 產出的安裝檔
- Post-build: 產生 SHA256 checksum，上傳至 GitHub Releases

### Install Script: `install.sh`

```bash
#!/bin/sh
# 1. 偵測 OS (darwin/linux) 和 arch (arm64/x86_64)
# 2. 組合下載 URL: github.com/AsiaOstrich/code-buddy/releases/latest/download/...
# 3. 下載 + 驗證 checksum
# 4. 解壓縮到 ~/.local/bin/
# 5. 確認 ~/.local/bin 在 PATH 中
# 6. 輸出成功訊息
```

### Homebrew Formula: `AsiaOstrich/homebrew-tap`

```
homebrew-tap/
├── Formula/
│   └── code-buddy.rb    # Homebrew Formula
└── README.md
```

Formula 核心結構：
- `url`: 指向 GitHub Releases 的 tar.gz
- `sha256`: 對應 checksum
- `depends_on`: 無額外依賴（預編譯 binary）

### 檔名規則

```
code-buddy-v{version}-{os}-{arch}.{ext}

os:   darwin | linux | windows
arch: arm64 | x86_64
ext:  tar.gz (macOS/Linux) | zip (Windows)
```

### CLI 擴充

現有 Tauri app 需新增 CLI 模式：

| 指令 | 功能 |
|------|------|
| `code-buddy --version` | 輸出版本號 |
| `code-buddy setup` | 自動設定 Claude Code hook |
| `code-buddy` (無參數) | 正常啟動 GUI |

## Risks

| 風險 | 影響 | 緩解策略 |
|------|------|---------|
| macOS 簽章/公證 | 使用者下載後被 Gatekeeper 攔截 | Tauri build 內建 codesign + notarize 支援 |
| Linux ARM64 CI runner | GitHub Actions 無原生 ARM64 runner | 使用 cross-compilation 或 QEMU |
| Windows Defender 誤判 | 未簽章 binary 被標記為不安全 | 考慮 Windows code signing certificate |
| 大檔案在 GitHub Releases | 每個平台 ~10-20MB | 可接受，GitHub Releases 無大小限制 |

## Milestones

```
v0.3.0-alpha  CI/CD pipeline（AC-1~4）
v0.3.0-beta   curl install script（AC-5~9）
v0.3.0        Homebrew tap（AC-10~12）
```

## Out of Scope

- npm global package 發行
- cargo install 發行
- 自動更新機制（app 內建檢查更新）
- GUI 安裝精靈
- Scoop (Windows) / AUR (Arch Linux) / Snap / Flatpak
- macOS App Store / Windows Store 上架
