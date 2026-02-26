# Deployment — Code Buddy

> Version: 0.2.0 | Updated: 2026-02-26

## Environment Requirements

### 必要工具

| 工具 | 最低版本 | 用途 | 安裝方式 |
|------|---------|------|---------|
| **Node.js** | 18+ | 前端建置、npm 套件管理 | [nodejs.org](https://nodejs.org/) 或 `brew install node` |
| **Rust** | stable | 後端編譯 | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` |
| **jq** | 1.6+ | Hook 腳本 JSON 解析 | `brew install jq` |

### 平台需求

| 平台 | 額外需求 |
|------|---------|
| **macOS** | Xcode Command Line Tools（`xcode-select --install`） |
| **Windows** | [規劃中] |
| **Linux** | [規劃中] |

### 驗證前置環境

```bash
node --version    # 應為 v18.x 或以上
cargo --version   # 應為 stable 版本
jq --version      # 應為 1.6 或以上
```

## Installation Steps

### 1. 取得原始碼

```bash
git clone https://github.com/AsiaOstrich/code-buddy.git
cd code-buddy
```

### 2. 安裝前端依賴

```bash
npm install
```

### 3. 開發模式啟動

```bash
npm run tauri dev
```

首次執行會編譯 Rust 後端（約 2-5 分鐘），後續啟動僅重新編譯變更部分。

### 4. 建置正式版

```bash
npm run tauri build
```

產出的安裝檔位於 `src-tauri/target/release/bundle/`。

## Configuration

### Claude Code Hook 設定

Code Buddy 透過 Claude Code 的 hook 機制接收事件。需在 Claude Code 設定中註冊 `scripts/claude-hook.sh`。

#### 步驟 1：確認腳本可執行

```bash
chmod +x scripts/claude-hook.sh
```

#### 步驟 2：設定 Claude Code hooks

在 `~/.claude/settings.json`（全域）或專案的 `.claude/settings.json` 中加入：

```json
{
  "hooks": {
    "UserPromptSubmit": [
      { "command": "/path/to/code-buddy/scripts/claude-hook.sh" }
    ],
    "PostToolUse": [
      { "command": "/path/to/code-buddy/scripts/claude-hook.sh" }
    ],
    "PostToolUseFailure": [
      { "command": "/path/to/code-buddy/scripts/claude-hook.sh" }
    ],
    "Stop": [
      { "command": "/path/to/code-buddy/scripts/claude-hook.sh" }
    ],
    "Notification": [
      { "command": "/path/to/code-buddy/scripts/claude-hook.sh" }
    ],
    "SessionStart": [
      { "command": "/path/to/code-buddy/scripts/claude-hook.sh" }
    ],
    "SessionEnd": [
      { "command": "/path/to/code-buddy/scripts/claude-hook.sh" }
    ]
  }
}
```

> 請將 `/path/to/code-buddy/` 替換為實際的專案路徑。

#### 環境變數

| 變數 | 預設值 | 說明 |
|------|-------|------|
| `CODE_BUDDY_PORT` | `19199` | HTTP server 監聽埠 |

### 開發模式選項

| 指令 | 說明 |
|------|------|
| `npm run tauri dev` | 前景開發模式（Vite HMR + Rust 熱編譯） |
| `npm run tauri:bg` | 背景開發模式（日誌輸出至 `/tmp/code-buddy-dev.log`） |
| `npm run tauri:stop` | 停止背景開發伺服器 |

## Verification

### 確認應用啟動

1. 執行 `npm run tauri dev`
2. 確認系統匣出現 Code Buddy 圖示（灰色鴕鳥 = Idle 狀態）
3. Debug 模式下會自動開啟 Dev Panel 視窗

### 確認 HTTP Server

```bash
curl http://localhost:19199/health
```

應回傳 `200 OK`。

### 確認 Hook 轉發

```bash
echo '{"hook_event_name":"SessionStart","session_id":"test-123","cwd":"/tmp"}' \
  | bash scripts/claude-hook.sh
```

Dev Panel 應顯示新的 test session。

### 確認通知權限

macOS 首次執行時會請求通知權限，請點選「允許」。可在「系統設定 → 通知」中確認 Code Buddy 已啟用。

## Troubleshooting

### Rust 編譯失敗

**症狀**：`cargo build` 或 `npm run tauri dev` 編譯錯誤。

```bash
# 確認 Rust toolchain 為最新
rustup update stable

# 清除編譯快取重試
cd src-tauri && cargo clean && cd ..
npm run tauri dev
```

### Hook 未生效

**症狀**：Claude Code 執行時 Code Buddy 無反應。

1. 確認 Code Buddy 已啟動（系統匣有圖示）
2. 確認 HTTP server 正常：`curl http://localhost:19199/health`
3. 確認 hook 腳本可執行：`ls -la scripts/claude-hook.sh`
4. 確認 `jq` 已安裝：`which jq`
5. 手動測試 hook 轉發（見上方「確認 Hook 轉發」）

### Port 衝突

**症狀**：HTTP server 啟動失敗，port 19199 已被占用。

```bash
# 查看佔用 port 的程序
lsof -i :19199

# 使用自訂 port
CODE_BUDDY_PORT=19200 npm run tauri dev
```

> 注意：同時需要更新 hook 腳本的環境變數 `CODE_BUDDY_PORT`。

### macOS 通知未顯示

1. 前往「系統設定 → 通知」
2. 找到 Code Buddy，確認通知已開啟
3. 確認「勿擾模式」未啟用

### Vite Dev Server 無法啟動

**症狀**：port 1420 已被占用。

```bash
# 查看佔用 port 的程序
lsof -i :1420

# 終止後重試
kill -9 <PID>
npm run tauri dev
```
