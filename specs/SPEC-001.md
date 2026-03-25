# SPEC-001 Code Buddy — AI Coding Agent 狀態監控桌面應用

> **Status**: Approved
> **Author**: AlbertHsu
> **Created**: 2026-02-26
> **Last Updated**: 2026-02-26
> **Approved**: 2026-02-26

---

## Summary

Code Buddy 是一款輕量桌面應用，透過常駐系統匣（System Tray）的動畫角色，即時視覺化 AI coding agent（Claude Code、OpenCode 等）的運行狀態，並在關鍵時刻推送桌面通知，讓開發者無需反覆切換視窗即可掌握 agent 動態。

## Motivation

- 開發者使用 AI coding agent 時經常切換視窗做其他事
- 無法即時感知 agent 狀態：還在跑？卡住了？等我回應？完成了？
- 只能反覆切回終端確認，嚴重打斷工作流
- 缺乏統一的多 agent / 多 session 監控方式

## Detailed Design

### Architecture Overview

```
                    ┌───────────────────┐
                    │   Claude Code     │
                    │  (Terminal Agent) │
                    └────────┬──────────┘
                             │ Hook 腳本
                             │ HTTP POST (JSON)
                             ▼
┌─────────────────────────────────────────────────┐
│                   Tauri v2 App                  │
├─────────────┬───────────────────────────────────┤
│  Rust Core  │         React Frontend            │
│             │                                   │
│  ┌────────┐ │  ┌──────────┐  ┌───────────────┐  │
│  │ HTTP   │ │  │  Status  │  │   Lottie      │  │
│  │ Server │ │  │  Panel   │  │   Animation   │  │
│  │ :3456  │ │  └──────────┘  └───────────────┘  │
│  ├────────┤ │  ┌──────────┐  ┌───────────────┐  │
│  │ System │ │  │  Session │  │   Settings    │  │
│  │ Tray   │ │  │  Manager │  │   Panel       │  │
│  ├────────┤ │  └──────────┘  └───────────────┘  │
│  │ Notif  │ │                                   │
│  │ Engine │ │                                   │
│  └────────┘ │                                   │
├─────────────┴───────────────────────────────────┤
│              Adapter Layer (State Machines)      │
│  ┌──────────────────┐  ┌──────────────────────┐ │
│  │ Claude Code      │  │ OpenCode             │ │
│  │ Adapter          │  │ Adapter              │ │
│  │ (Hooks→HTTP)     │  │ (SSE /global/event)  │ │
│  └──────────────────┘  └──────────────────────┘ │
└─────────────────────────────────┬───────────────┘
                                  │ SSE Stream
                                  ▼
                    ┌───────────────────┐
                    │    OpenCode       │
                    │  Server :4096     │
                    └───────────────────┘
```

### Agent States

| State | 說明 | Tray 圖示 | Dock 圖示 | 面板動畫（鴕鳥） | 顏色 | 通知 |
|-------|------|----------|----------|-----------------|------|------|
| `working` | Agent 正在執行工具/寫程式碼 | 🔵 藍色圓點 | 鴕鳥 + 右下🔵 | 鴕鳥快速跑步/敲鍵盤 | 藍色 | - |
| `thinking` | Agent 正在思考/推理 | 🟣 紫色圓點 | 鴕鳥 + 右下🟣 | 鴕鳥歪頭 + 頭上冒泡泡 | 紫色 | - |
| `waiting_input` | Agent 等待使用者輸入 | 🟡 黃色圓點 | 鴕鳥 + 右下🟡 | 鴕鳥招手/敲門動作 | 黃色 | 推送通知 |
| `completed` | 任務完成 | 🟢 綠色圓點 | 鴕鳥 + 右下🟢 | 鴕鳥慶祝跳躍 | 綠色 | 推送通知 |
| `error` | 發生錯誤 | 🔴 紅色圓點 | 鴕鳥 + 右下🔴 | 鴕鳥把頭埋進沙裡 | 紅色 | 推送通知 |
| `idle` | Agent 閒置/未連線 | ⚪ 灰色圓點 | 鴕鳥 + 右下⚪ | 鴕鳥打盹/慢速呼吸 | 灰色 | - |
| `waiting_confirm` | 等待使用者確認操作 | 🟠 橙色圓點 | 鴕鳥 + 右下🟠 | 鴕鳥舉手/舉牌詢問 | 橙色 | 推送通知 |

### Adapter Architecture

每個 AI agent 透過獨立 adapter 對接，實作統一的 `AgentAdapter` trait。所有 adapter 邏輯在 Rust 端實作（Rust-heavy 架構）。

#### Rust 端核心型別（Adapter Layer）

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentType {
    ClaudeCode,
    OpenCode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum AgentStatus {
    WaitingInput = 0,     // 最高優先：使用者需要行動
    WaitingConfirm = 1,
    Error = 2,
    Working = 3,
    Thinking = 4,
    Completed = 5,
    Idle = 6,             // 最低優先
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentState {
    pub session_id: String,
    pub agent_type: AgentType,
    pub status: AgentStatus,
    pub message: Option<String>,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionInfo {
    pub session_id: String,
    pub agent_type: AgentType,
    pub project_name: String,       // cwd 最後一段目錄名
    pub project_path: String,       // 完整 cwd 路徑
    pub status: AgentStatus,
    pub last_updated: u64,          // 最近狀態變化時間戳（epoch ms）
    pub duration: u64,              // session 持續時間（ms）
    pub pinned: bool,               // 是否被使用者釘選為焦點
}

/// Adapter trait — 每種 agent 各自實作
#[async_trait::async_trait]
pub trait AgentAdapter: Send + Sync {
    fn agent_type(&self) -> AgentType;
    async fn start(&self, state: Arc<Mutex<AppState>>, app: AppHandle) -> Result<()>;
    async fn stop(&self) -> Result<()>;
}
```

#### 前端 TypeScript 型別（純 UI 渲染用）

前端透過 Tauri IPC 接收 Rust 序列化的 JSON，以下為對應的 TypeScript 型別定義：

```typescript
type AgentType = 'claude_code' | 'open_code';

type AgentStatus =
  | 'working'
  | 'thinking'
  | 'waiting_input'
  | 'completed'
  | 'error'
  | 'idle'
  | 'waiting_confirm';

interface SessionInfo {
  sessionId: string;
  agentType: AgentType;
  projectName: string;
  projectPath: string;
  status: AgentStatus;
  lastUpdated: number;
  duration: number;
  pinned: boolean;
}

/** 狀態優先順序（高 → 低），用於前端排序顯示 */
const STATUS_PRIORITY: AgentStatus[] = [
  'waiting_input',
  'waiting_confirm',
  'error',
  'working',
  'thinking',
  'completed',
  'idle',
];
```

### Claude Code Adapter

- **偵測方式**：Hooks → HTTP 轉發至 Code Buddy 內建 HTTP server
- **通訊流程**：`Claude Code → Hook 腳本 → HTTP POST → Code Buddy (Tauri)`
- **資料格式**：JSON via stdin（Hook 接收）→ JSON via HTTP（轉發）

#### 使用的 Hook 事件

| Hook 事件 | 對應 Code Buddy 狀態 | 說明 |
|-----------|---------------------|------|
| `SessionStart` | `idle` → 開始追蹤 | session 啟動，註冊新 session |
| `UserPromptSubmit` | `thinking` | 使用者送出指令，agent 開始思考（見 thinking 偵測） |
| `PostToolUse` | `working` | 工具執行後觸發，表示 agent 正在工作 |
| `Stop` | `completed` | agent 完成回應 |
| `Notification` (idle_prompt) | `waiting_input` | agent 等待使用者輸入 |
| `Notification` (permission_prompt) | `waiting_confirm` | agent 等待權限確認 |
| `PostToolUseFailure` | `working`（附加 warning） | 單次工具失敗，agent 可能繼續嘗試（見防抖機制） |
| `SessionEnd` | 移除 session | session 結束 |
| `SubagentStart` / `SubagentStop` | 子 agent 追蹤 | 監控子代理生命週期 |

> **Note**: Notification 還有 `auth_success` 和 `elicitation_dialog` 兩種 matcher，MVP 暫不使用。

#### Hook 設定範例

使用獨立腳本 `scripts/claude-hook.sh` 統一轉發事件，方便維護與錯誤處理：

```json
{
  "hooks": {
    "PostToolUse": [{
      "matcher": "",
      "hooks": [{ "type": "command", "command": "scripts/claude-hook.sh", "async": true, "timeout": 5 }]
    }],
    "UserPromptSubmit": [{
      "hooks": [{ "type": "command", "command": "scripts/claude-hook.sh", "async": true }]
    }],
    "Stop": [{
      "hooks": [{ "type": "command", "command": "scripts/claude-hook.sh", "async": true }]
    }],
    "Notification": [{
      "hooks": [{ "type": "command", "command": "scripts/claude-hook.sh", "async": true }]
    }],
    "SessionStart": [{
      "hooks": [{ "type": "command", "command": "scripts/claude-hook.sh", "async": true }]
    }],
    "SessionEnd": [{
      "hooks": [{ "type": "command", "command": "scripts/claude-hook.sh", "async": true }]
    }],
    "PostToolUseFailure": [{
      "hooks": [{ "type": "command", "command": "scripts/claude-hook.sh", "async": true }]
    }]
  }
}
```

#### `scripts/claude-hook.sh` 腳本內容

```bash
#!/bin/bash
# Code Buddy — Claude Code Hook 轉發腳本
# 將 Hook 事件 JSON 透過 HTTP POST 轉發至 Code Buddy
CODE_BUDDY_URL="${CODE_BUDDY_URL:-http://localhost:3456/claude-code/event}"
curl -s -X POST "$CODE_BUDDY_URL" \
  -H 'Content-Type: application/json' \
  -d "$(cat)" \
  --connect-timeout 2 \
  --max-time 5 \
  2>/dev/null || true  # 靜默失敗，不阻塞 agent
```

#### Hook JSON 輸入格式（Claude Code 提供）

所有 Hook 事件共用以下 common fields：

```json
{
  "session_id": "abc123",
  "transcript_path": "/Users/user/.claude/projects/.../transcript.jsonl",
  "cwd": "/Users/user/myproject",
  "permission_mode": "default",
  "hook_event_name": "PostToolUse",
  "tool_name": "Bash",
  "tool_input": { "command": "npm test" }
}
```

> **Note**: `tool_name` 和 `tool_input` 僅在 tool 相關事件（`PreToolUse`、`PostToolUse`、`PostToolUseFailure`、`PermissionRequest`）中出現。Notification 事件另有 `message`、`title`、`notification_type` 欄位。

#### 狀態防抖機制

為避免 `PostToolUseFailure` 導致狀態在 `error` ↔ `working` 之間快速閃爍：

| 規則 | 說明 |
|------|------|
| **單次失敗** | 維持 `working` 狀態，附加 `lastWarning` 訊息 |
| **連續 3 次失敗**（同一 session，60s 內） | 轉為 `error` 狀態 |
| **Stop 事件確認** | 若 `Stop` 事件的 `stop_reason` 為 error 類型，直接轉為 `error` |
| **新 PostToolUse 成功** | 清除失敗計數器，恢復 `working` |

#### `thinking` 狀態偵測

Claude Code 沒有直接對應 `thinking` 的 Hook 事件。採用以下推導策略：

| 條件 | 推導結果 |
|------|---------|
| 收到 `UserPromptSubmit` 後，尚未收到任何 `PostToolUse` | `thinking`（agent 正在規劃） |
| 收到 `PostToolUse` 後超過 5 秒無新事件 | `thinking`（agent 在 tool 之間思考） |
| 收到下一個 `PostToolUse` | 回到 `working` |

> **Note**: Claude Code 還有 `UserPromptSubmit`、`PreToolUse`、`PermissionRequest` 等 Hook 事件（共 17 個），MVP 僅使用上表列出的 8 個，其餘可在後續版本擴充。

#### 補充資訊來源：StatusLine

StatusLine 提供額外的 session metadata（model、cost、context_window），可透過獨立腳本定期寫入共享檔案，供 Code Buddy 讀取以豐富 UI 顯示。更新頻率：300ms 防抖。

### OpenCode Adapter

- **偵測方式**：SSE `/global/event` 直連（零侵入）
- **通訊流程**：`OpenCode Server → SSE Stream → Code Buddy (Tauri)`
- **Server 位址**：偏好 `http://localhost:4096/global/event`（TUI 啟動時自動開啟）。若 4096 埠被佔用，OpenCode 會回退到隨機埠。Code Buddy 透過掃描 `~/.opencode/` 下的 lock file 或使用者手動指定來發現實際埠號。
- **認證**：HTTP Basic Auth（如有設定 `OPENCODE_SERVER_PASSWORD`）
- **CORS**：官方已內建 `tauri://localhost` 支援

#### SSE 事件對應

| OpenCode 事件 | 對應 Code Buddy 狀態 | 說明 |
|--------------|---------------------|------|
| `session.status: busy` | `working` | agent 正在處理 |
| `message.part.updated`（busy 中） | `thinking` | 正在生成回應 |
| `session.status: idle` | `waiting_input` 或 `completed` | 見下方 idle 判斷規則 |
| `session.status: retry` | `error` | 遇到錯誤，自動重試中 |
| `session.error` | `error` | 處理失敗 |
| `permission.asked` | `waiting_confirm` | 等待權限確認 |
| `server.heartbeat` | 連線存活 | 每 10 秒一次，超時視為斷線 |

#### `idle` 狀態判斷規則

OpenCode 的 `session.status: idle` 無法直接區分「任務完成」與「等待輸入」，需結合上下文：

| 條件 | 判斷結果 | 說明 |
|------|---------|------|
| `idle` 前最後事件為 `message.part.updated` 或 `message.updated` | `completed` | agent 已回覆完畢 |
| `idle` 前最後事件為 `session.created` | `waiting_input` | 新 session，等待第一條指令 |
| `idle` 前最後事件為 `permission.replied` | `completed` | 權限處理完畢 |
| `idle` 且超過 30 秒無其他事件 | `idle` | 真正閒置 |
| 其他情況 | `completed`（預設） | 安全預設值 |

> **Note**: OpenCode SSE 還提供 `session.created`、`session.deleted`、`permission.replied` 等事件，可在 Post-MVP 中進一步利用。

#### SSE 事件資料格式

```json
{
  "directory": "/path/to/project",
  "payload": {
    "type": "session.status",
    "properties": {
      "sessionID": "abc123",
      "status": { "type": "busy" }
    }
  }
}
```

#### OpenCode SessionStatus 型別

```typescript
type SessionStatus =
  | { type: 'idle' }
  | { type: 'busy' }
  | { type: 'retry'; attempt: number; message: string; next: number };
```

### 統一狀態機設計

兩個 adapter 各自將原始信號轉為統一的 `AgentState`，前端只關心統一狀態：

```
┌──────────────┐     ┌─────────────────┐     ┌──────────────┐
│ Claude Code  │────▶│  State Machine  │────▶│   Unified    │
│ Hook Events  │     │  (per adapter)  │     │  AgentState  │
└──────────────┘     └─────────────────┘     └──────────────┘
┌──────────────┐     ┌─────────────────┐          │
│  OpenCode    │────▶│  State Machine  │──────────┘
│  SSE Events  │     │  (per adapter)  │
└──────────────┘     └─────────────────┘
```

### Cold Start / Late Join 處理

Code Buddy 可能在 agent 已運行後才啟動（或重啟）。各 adapter 的處理策略：

| 場景 | Claude Code | OpenCode |
|------|------------|----------|
| **Agent 已運行** | 無法回溯歷史事件；等待下一個 Hook 事件觸發後才開始追蹤 | SSE 連線時收到 `server.connected` 初始事件，可查詢 `/session` API 取得現有 session 列表 |
| **Code Buddy 重啟** | 已知 session 全部重置為 `idle`；下一個 Hook 事件自動恢復正確狀態 | 重新建立 SSE 連線，reqwest-eventsource 自動重連 |
| **Code Buddy 未啟動時** | Hook 的 `curl` 靜默失敗（`async: true` 不阻塞 agent），事件丟失 | 無影響（SSE 尚未連線） |

> **MVP 策略**：接受 Cold Start 的狀態遺漏，以簡化實作。Tray 顯示 `idle` 直到收到第一個事件。後續可考慮 Claude Code 的 `transcript_path` 回溯歷史。

### Code Buddy 內建 HTTP Server

Tauri Rust backend 啟動一個本地 HTTP server（預設 `localhost:3456`），負責：

1. **接收 Claude Code Hook 事件**：`POST /claude-code/event`
2. **健康檢查**：`GET /health`
3. **Session 列表**：`GET /sessions`

此 server 僅綁定 `127.0.0.1`，不對外暴露。

### Character Design — 角色設計

#### 角色：小鴕鳥（AsiaOstrich 品牌 IP）

選擇鴕鳥作為 Buddy 角色的理由：
- 與 AsiaOstrich 品牌一致，強化 IP 識別
- 鴕鳥動作天然適合各種狀態（跑步、歪頭、埋頭、招手）
- 「error 時把頭埋進沙裡」具記憶點，增加情感連結
- 獨特輪廓在 22px Menu Bar 中仍可辨識

#### Tray 圖示策略：純色圓點

macOS Menu Bar 圖示（~22x22px）採用純色圓點，不含角色輪廓：
- 極簡設計，在小尺寸下清晰可辨
- 使用者可直覺理解（綠點=完成、黃點=需回應）
- 技術實作簡單（靜態 PNG 切換），跨平台相容性好

#### Dock 圖示策略：鴕鳥輪廓 + 狀態圓點

macOS Dock 圖示採用鴕鳥角色輪廓 + 右下角彩色狀態圓點：
- 類似 Slack/Discord 的在線狀態指示器
- Dock 圖示空間充足（128x128+），適合展示角色識別
- 強化 AsiaOstrich 品牌 IP，角色在 Dock 中一眼可辨
- 透過 `NSApplication::setApplicationIconImage` 動態切換

#### 主題包系統

角色動畫設計為可替換的「主題包」，MVP 內建鴕鳥主題，未來支援社群貢獻：

```
assets/themes/
├── ostrich/                # 預設主題（鴕鳥）
│   ├── theme.json          # 主題 metadata（名稱、作者、版本）
│   ├── tray/               # Tray 圖示（每種狀態一個 PNG）
│   │   ├── working.png
│   │   ├── thinking.png
│   │   ├── waiting_input.png
│   │   ├── completed.png
│   │   ├── error.png
│   │   ├── idle.png
│   │   └── waiting_confirm.png
│   └── animations/         # Lottie 面板動畫（每種狀態一個 JSON）
│       ├── working.json
│       ├── thinking.json
│       ├── waiting_input.json
│       ├── completed.json
│       ├── error.json
│       ├── idle.json
│       └── waiting_confirm.json
└── (future themes)/        # 未來擴充：pixel-pet、robot 等
```

#### theme.json 格式

```json
{
  "id": "ostrich",
  "name": "小鴕鳥",
  "version": "1.0.0",
  "author": "AsiaOstrich",
  "description": "Code Buddy 預設鴕鳥主題",
  "states": {
    "working":         { "tray": "tray/working.png",         "animation": "animations/working.json" },
    "thinking":        { "tray": "tray/thinking.png",        "animation": "animations/thinking.json" },
    "waiting_input":   { "tray": "tray/waiting_input.png",   "animation": "animations/waiting_input.json" },
    "completed":       { "tray": "tray/completed.png",       "animation": "animations/completed.json" },
    "error":           { "tray": "tray/error.png",           "animation": "animations/error.json" },
    "idle":            { "tray": "tray/idle.png",            "animation": "animations/idle.json" },
    "waiting_confirm": { "tray": "tray/waiting_confirm.png", "animation": "animations/waiting_confirm.json" }
  }
}
```

#### 動畫設計原則

| 原則 | 說明 |
|------|------|
| **輕量** | 每個 Lottie JSON < 50KB，總主題包 < 500KB |
| **循環** | working / thinking / idle 為無限循環動畫 |
| **一次性** | completed / error 播放一次後靜止 |
| **節奏感** | working（快節奏）> thinking（中等）> idle（慢呼吸） |
| **高對比** | 顏色在淺色/深色 Menu Bar 都清晰可見 |

### Multi-Session Management — 多 Session 管理

#### Tray 聚合策略：最高優先狀態 + 數字徽章

多 session 同時運行時，Tray 圖示顯示所有 session 中**最需要注意的狀態**，並在右上角以數字徽章標示「需要注意」的 session 數量：

```
[🐦🟢]    ← 全部正常（working/completed/idle）
[🐦🟡]    ← 有 1 個 session 需要注意
[🐦🟡②]  ← 有 2 個 session 需要注意
[🐦🔴①]  ← 有 1 個 session 出錯
```

狀態優先順序（高 → 低）：

```
waiting_input    ←── 最高（使用者需要行動）
waiting_confirm
error
working
thinking
completed
idle             ←── 最低
```

「需要注意」的定義：狀態為 `waiting_input`、`waiting_confirm` 或 `error`。

#### Session 身份識別

```
格式: [狀態顏色] [Agent 圖示] [專案名稱]

範例:
  🟡 🤖 my-app         ← Claude Code, my-app 專案, waiting_input
  🔵 🔷 backend-api    ← OpenCode, backend-api 專案, working

Agent 圖示:
  🤖 = Claude Code
  🔷 = OpenCode

專案名稱來源:
  Claude Code → hook_event.cwd 的最後一段目錄名
  OpenCode   → SSE event.directory 的最後一段目錄名

重複目錄名處理:
  若有兩個 session 都叫 "my-app":
  → 🟡 🤖 my-app (abc1)
  → 🔵 🤖 my-app (def2)
  括號內顯示 sessionId 前 4 碼
```

#### Session 列表排序

列表依「需要注意程度」分組排序，閒置 session 聚合收合：

```
┌────────────────────────┐
│  📋 Sessions           │
│                        │
│  ⚠️ 需要注意 (2)       │
│  ┌──────────────────┐  │
│  │ 🟡 🤖 project-a  │  │  ← waiting_input，自動置頂
│  │    等待輸入 2m    │  │
│  ├──────────────────┤  │
│  │ 🔴 🤖 project-b  │  │  ← error，自動置頂
│  │    執行失敗 30s   │  │
│  └──────────────────┘  │
│                        │
│  🔵 工作中 (1)         │
│  ┌──────────────────┐  │
│  │ 🔵 🔷 project-c  │  │  ← working
│  │    工作中 5m      │  │
│  └──────────────────┘  │
│                        │
│  ⚪ 閒置 (3)    ▶      │  ← 聚合，點擊展開
└────────────────────────┘
```

分組規則：
- **需要注意**：`waiting_input` / `waiting_confirm` / `error`
- **工作中**：`working` / `thinking`
- **已完成**：`completed`
- **閒置**：`idle`（3+ 時聚合為一行，點擊展開）

#### 焦點 Session 切換

面板上方的動畫區顯示「焦點 session」的 Lottie 動畫：

| 場景 | 行為 |
|------|------|
| **無釘選**（預設） | 自動跟隨最近有狀態變化的 session |
| **有釘選** | 固定顯示被釘選的 session |
| **點擊通知** | 暫時切到該 session（不影響釘選） |
| **點擊列表項** | 切到該 session 並可選釘選 |

#### Session 自動清理

- 閒置超過 30 分鐘的 session 自動標記為「過期」
- 過期 session 在聚合列表中不計入數字
- 使用者可手動移除 session

### UX Interaction Design — 互動流程設計

#### 三層感知模型

Code Buddy 提供三個層次的狀態感知，使用者可依需求選擇深度：

```
被動感知 ───────────── 主動查看 ───────────── 持續監控
(零操作)              (按需點擊)             (進階模式)

Menu Bar             Popover 面板           浮動小視窗
[🐦🟢]               點擊展開               always-on-top
顏色點即時變化         動畫+列表              半透明常駐
                     點擊外部關閉

 MVP ◄──────────────── MVP ──────────────► Post-MVP
```

#### 互動方式

| 操作 | 行為 | 範圍 |
|------|------|------|
| **左鍵點擊** Tray 圖示 | 展開/收起 Popover 面板（toggle） | MVP |
| **右鍵點擊** Tray 圖示 | Context Menu（靜音、設定、退出） | MVP |
| **點擊面板外部** | 自動關閉 Popover（macOS 標準行為） | MVP |
| **ESC** | 關閉 Popover | MVP |
| **點擊通知** | 展開 Popover 面板並高亮對應 session | MVP |
| **拖拉浮動視窗** | 移動位置，記住上次位置 | Post-MVP |

#### Popover 面板 Wireframe（MVP）

```
┌────────────────────────┐
│    [鴕鳥動畫 120x120]   │  ← 焦點 session 的 Lottie 動畫
│                        │
│  🟡 🤖 project-a       │  ← Agent 圖示 + 專案名
│  等待輸入               │  ← 狀態文字
│  3m 42s                │  ← 持續時間
├────────────────────────┤
│  📋 Sessions           │
│                        │
│  ⚠️ 需要注意 (1)       │
│  ┌──────────────────┐  │
│  │ 🟡 🤖 project-a 📌│  │  ← 釘選中，waiting_input
│  │    等待輸入 3m    │  │
│  └──────────────────┘  │
│                        │
│  🔵 工作中 (2)         │
│  ┌──────────────────┐  │
│  │ 🔵 🤖 project-b  │  │
│  │    工作中 5m      │  │
│  ├──────────────────┤  │
│  │ 🔵 🔷 project-c  │  │
│  │    工作中 12m     │  │
│  └──────────────────┘  │
│                        │
│  ⚪ 閒置 (2)    ▶      │  ← 點擊展開
└────────────────────────┘
  面板大小: ~300 x 420px
```

#### 右鍵 Context Menu

```
┌─────────────────────┐
│ 靜音通知 30 分鐘     │
│ 設定...              │
│ ───────────────────  │
│ 關於 Code Buddy      │
│ 退出                 │
└─────────────────────┘
```

#### 通知策略

MVP 採用「僅關鍵通知」，後續升級為分級系統：

| 級別 | 呈現方式 | 觸發狀態 | 範圍 |
|------|----------|---------|------|
| **關鍵** | macOS 系統通知 + 音效 | `waiting_input`, `waiting_confirm` | MVP |
| **重要** | macOS 系統通知（靜音） | `completed`, `error` | MVP |
| **資訊** | 僅 Tray 圖示顏色變化 | `working` ↔ `thinking` | MVP |
| **靜默** | 僅面板內記錄 | 心跳、狀態微調 | Post-MVP |

#### 完整互動流程

```
使用者啟動 Agent
       │
       ▼
 Code Buddy 自動偵測 (Hook/SSE)
       │
       ▼
┌──────────────────────────────────────┐
│       被動感知（零操作）               │
│  Menu Bar: [🐦🔵] ← 顏色點即時變化    │
│  餘光即可感知 agent 狀態               │
└──────────┬───────────────────────────┘
           │
  ┌────────┴─────────┐
  │                  │
狀態變化            使用者好奇
(completed/         (想看細節)
 waiting_input)          │
  │                  │
  ▼                  ▼
系統通知          左鍵點擊 Tray
推送到桌面             │
  │                  ▼
  │            Popover 面板
  │            動畫 + Session 列表
  │                  │
  │            點擊外部 / ESC 關閉
  ▼                  ▼
繼續工作          繼續工作
```

#### 浮動小視窗模式（Post-MVP）

獨立於 Tray Popover 之外的第三種顯示模式，適合多螢幕使用者：

| 屬性 | 說明 |
|------|------|
| **定位** | always-on-top，可自由拖拉，記住上次位置 |
| **透明度** | 可調（30%~100%），預設 70%，降低視覺干擾 |
| **大小** | 小（80x80 僅角色）/ 中（160x200 角色+狀態文字） |
| **內容** | 當前焦點 session 的 Lottie 動畫 + 狀態顏色邊框 |
| **互動** | 點擊浮動視窗 → 展開完整 Popover 面板 |
| **啟用方式** | 設定面板中開啟，或右鍵 Menu → 浮動模式 |

```
浮動小視窗 wireframe（小尺寸 80x80）:
┌──────────┐
│  🐦      │  ← Lottie 動畫
│  ┌─🟢    │  ← 右下角狀態顏色點
└──────────┘
  透明度 70%
  always-on-top

浮動小視窗 wireframe（中尺寸 160x200）:
┌────────────────┐
│                │
│   🐦 鴕鳥      │  ← Lottie 動畫
│   慶祝跳躍     │
│                │
│  ✅ completed  │  ← 狀態文字
│  project-a     │  ← session 名稱
└────────────────┘
  透明度 70%
  always-on-top
```

### Tech Stack

| 層級 | 技術 | 理由 |
|------|------|------|
| 桌面框架 | Tauri v2 (Rust) | 輕量（~10MB），跨平台，原生 system tray |
| HTTP server | axum | 與 Tauri 共用 tokio runtime，零額外開銷 |
| SSE client | reqwest-eventsource | 自動重連，生態成熟（238 萬+下載） |
| 前端 | React + TypeScript | 生態成熟 |
| 前端狀態 | Zustand | 輕量（<1KB），無 boilerplate，事件驅動更新 |
| 動畫 | Lottie (lottie-react) | 向量動畫，檔案小，易替換角色 |
| Tray 定位 | tauri-plugin-positioner | 官方插件，Popover 定位到 Tray 下方 |
| 通知 | tauri-plugin-notification | 官方插件，macOS/Windows 原生通知 |
| 跨平台 | macOS + Windows (+ Linux) | Tauri v2 原生支援 |

### Rust / React Responsibility Split — 職責劃分

採用 **Rust 重心型**架構：所有業務邏輯在 Rust 端，React 只做 UI 渲染。

```
┌─────────────────────────────────────────────────────────┐
│  Rust Backend（業務邏輯核心）                              │
│                                                         │
│  ┌──────────┐    ┌──────────────┐    ┌───────────────┐  │
│  │ HTTP     │───▶│              │───▶│ System Tray   │  │
│  │ Server   │    │  AppState    │    │ set_icon()    │  │
│  │ (axum)   │    │  (Mutex)     │    └───────────────┘  │
│  └──────────┘    │              │    ┌───────────────┐  │
│  ┌──────────┐    │ - sessions[] │───▶│ Notification  │  │
│  │ SSE      │───▶│ - focus_id   │    │ send()        │  │
│  │ Client   │    │ - settings   │    └───────────────┘  │
│  └──────────┘    │              │    ┌───────────────┐  │
│                  │              │───▶│ emit() Event  │  │
│                  └──────────────┘    │ → Frontend    │  │
│                    State Machine     └───────────────┘  │
├─────────────────────────────────────────────────────────┤
│  React Frontend（純 UI 渲染）                             │
│                                                         │
│  listen("state-changed") → Zustand store → Components   │
│  invoke("get_sessions") → 初始載入快照                    │
└─────────────────────────────────────────────────────────┘
```

| 層 | 職責 | 具體內容 |
|----|------|---------|
| **Rust** | 業務邏輯 | 狀態機轉換、session 管理、通知決策、Tray 圖示更新 |
| **Rust** | 系統 API | HTTP server（axum）、SSE client、System Tray、桌面通知 |
| **Rust** | 狀態管理 | `Mutex<AppState>` — 唯一真相來源 |
| **React** | UI 渲染 | Popover 面板、Lottie 動畫、Session 列表 |
| **React** | 前端狀態 | Zustand store — 接收 Rust 推送的快照 |

### IPC Design — 前後端通訊

採用 **Events（推送）+ Commands（拉取）混合模式**：

| 場景 | IPC 機制 | 方向 | 說明 |
|------|---------|------|------|
| Hook/SSE 事件進入 | `emit("state-changed")` | Rust → React | 即時推送狀態變化 |
| 前端首次載入 | `invoke("get_sessions")` | React → Rust | 拉取完整狀態快照 |
| 使用者釘選 session | `invoke("pin_session")` | React → Rust | 發送操作指令 |
| 使用者靜音通知 | `invoke("mute_notifications")` | React → Rust | 發送設定變更 |

#### 狀態變更事件 Payload

```rust
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct StateChangedEvent {
    session_id: String,           // 觸發變更的 session
    status: String,               // 新狀態
    project_name: String,
    agent_type: String,
    all_sessions: Vec<SessionInfo>, // 全量快照（簡化前端邏輯）
    tray_status: String,          // Tray 聚合狀態
    attention_count: usize,       // 需要注意的 session 數
}
```

MVP 推送全量快照（`all_sessions`），簡單可靠。後續效能瓶頸時再優化為差量更新。

#### Rust 端 AppState

```rust
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Default, Clone, Serialize)]
pub struct AppState {
    pub sessions: HashMap<String, SessionInfo>,
    pub focus_session_id: Option<String>,
    pub pinned_session_id: Option<String>,
    pub notification_muted_until: Option<u64>,
}
```

#### 前端 Zustand Store

```typescript
import { create } from 'zustand';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';

interface AppStore {
  sessions: SessionInfo[];
  focusSessionId: string | null;
  trayStatus: string;
  attentionCount: number;
  init: () => Promise<void>;
  pinSession: (id: string) => Promise<void>;
}

const useAppStore = create<AppStore>((set) => ({
  sessions: [],
  focusSessionId: null,
  trayStatus: 'idle',
  attentionCount: 0,

  init: async () => {
    // 1. 先建立 listener，避免快照拉取期間遺漏事件
    listen<StateChangedEvent>('state-changed', (event) => {
      set({
        sessions: event.payload.allSessions,
        trayStatus: event.payload.trayStatus,
        attentionCount: event.payload.attentionCount,
        focusSessionId: event.payload.sessionId,
      });
    });

    // 2. 再拉取初始快照（listener 已就緒，不會遺漏）
    const state = await invoke<StateChangedEvent>('get_sessions');
    set({ sessions: state.allSessions, trayStatus: state.trayStatus });
  },

  pinSession: async (id) => {
    await invoke('pin_session', { sessionId: id });
  },
}));
```

### Tauri v2 Key Decisions — 關鍵技術決策

| 決策 | 選擇 | 理由 |
|------|------|------|
| macOS Dock | `ActivationPolicy::Regular` | 顯示在 Dock，動態切換鴕鳥圖示 + 右下角狀態圓點 |
| 視窗建立 | 動態建立（非 tauri.conf.json） | Popover 按需建立，減少啟動資源 |
| Popover 關閉 | 監聽 `Focused(false)` 事件 | 點擊外部自動隱藏 |
| Popover 定位 | `Position::TrayBottomCenter` | tauri-plugin-positioner 提供 |
| HTTP 啟動 | `tauri::async_runtime::spawn()` | 與 Tauri 共用 tokio runtime |
| SSE 重連 | reqwest-eventsource 內建 | 自動重連，不需手動實作 |

### Directory Structure (Planned)

```
code-buddy/
├── src-tauri/                # Rust backend
│   ├── src/
│   │   ├── main.rs           # Tauri 啟動 + setup
│   │   ├── state.rs          # AppState 定義
│   │   ├── tray.rs           # System Tray 建立 + 圖示切換
│   │   ├── popover.rs        # Popover 視窗管理（toggle/定位）
│   │   ├── notification.rs   # 桌面通知（通知決策邏輯）
│   │   ├── server.rs         # axum HTTP server (:3456)
│   │   ├── commands.rs       # Tauri Commands（get_sessions, pin_session 等）
│   │   └── adapters/
│   │       ├── mod.rs        # Adapter trait + 狀態機 + 狀態優先順序
│   │       ├── claude_code.rs  # Hook 事件解析 → AppState 更新
│   │       └── opencode.rs     # SSE 連線 + 事件解析
│   ├── Cargo.toml
│   └── tauri.conf.json       # windows: []（動態建立）
├── src/                      # React frontend
│   ├── App.tsx
│   ├── store.ts              # Zustand store（listen + invoke）
│   ├── components/
│   │   ├── StatusPanel.tsx    # 動畫區 + 狀態文字
│   │   ├── SessionList.tsx    # Session 分組列表
│   │   ├── BuddyAnimation.tsx # Lottie 動畫渲染
│   │   └── Settings.tsx
│   └── assets/
│       └── themes/
│           └── ostrich/       # 預設鴕鳥主題
│               ├── theme.json
│               ├── tray/      # 7 種狀態 PNG
│               └── animations/ # 7 種狀態 Lottie JSON
├── scripts/
│   └── claude-hook.sh         # Claude Code Hook 轉發腳本
├── specs/
├── package.json
└── tsconfig.json
```

## Acceptance Criteria

### Milestone Roadmap

```
v0.1.0 Skeleton        v0.2.0 Detection       v0.3.0 Panel
┌──────────────┐      ┌──────────────┐       ┌──────────────┐
│ System Tray  │      │ HTTP Server  │       │ Popover 面板  │
│ 靜態圖示切換  │─────▶│ Hook 整合    │──────▶│ Lottie 動畫   │
│ Context Menu │      │ 狀態偵測     │       │ Session 資訊  │
│              │      │ 桌面通知     │       │              │
└──────────────┘      └──────────────┘       └──────────────┘
  技術驗證               核心價值驗證            完整體驗
```

### v0.1.0 — Skeleton（骨架）

**目標**：驗證 Tauri v2 + System Tray 的技術可行性

**驗證問題**：Tauri v2 能做出合格的 macOS Menu Bar 常駐應用嗎？
**成功指標**：應用常駐、圖示切換正常、< 20MB

- [ ] AC-1: 應用程式可在 macOS 上以 System Tray 常駐，同時顯示在 Dock
- [ ] AC-1.1: Dock 圖示顯示鴕鳥輪廓 + 右下角彩色狀態圓點，可隨狀態動態切換
- [ ] AC-2: System Tray 圖示可手動切換狀態（純色圓點 PNG），驗證 7 種狀態圖示顯示正確
- [ ] AC-3: 右鍵點擊 Tray 圖示顯示 Context Menu（退出、關於）
- [ ] AC-4: 應用大小 < 20MB，記憶體使用 < 100MB

### v0.2.0 — Detection（偵測）

**目標**：驗證核心價值「開發者可以被動感知 agent 狀態」

**驗證問題**：開發者看到 Tray 圖示變色 + 通知，會覺得有用嗎？
**成功指標**：自己日常使用 1 週後不想關掉

- [ ] AC-5: 可偵測本機至少一個 Claude Code session 的狀態（HTTP server + Hook 整合）
- [ ] AC-6: System Tray 圖示根據真實 Hook 事件即時切換（至少區分 working / idle / waiting_input / completed）
- [ ] AC-7: 狀態變為 `completed`、`waiting_input`、`waiting_confirm`、`error` 時推送 macOS 原生桌面通知

### v0.3.0 — Panel（面板）

**目標**：完整 MVP 體驗

**驗證問題**：加了面板和動畫後，使用者會主動點擊查看嗎？
**成功指標**：使用者會主動點擊查看面板

- [ ] AC-8: 左鍵點擊 Tray 圖示展開 Popover 面板，點擊外部或 ESC 關閉
- [ ] AC-9: 面板顯示當前 session 的狀態動畫（Lottie 或靜態 SVG）+ 狀態文字 + 持續時間
- [ ] AC-10: 右鍵 Context Menu 擴充（靜音通知 30 分鐘、設定）

### Post-MVP

- [ ] AC-11: 多 session 監控 — 同時追蹤多個 agent instance，Tray 顯示最高優先狀態 + 數字徽章，面板分組排序
- [ ] AC-11.1: Session 釘選 — 可釘選焦點 session，動畫區固定顯示
- [ ] AC-11.2: 閒置聚合 — 3+ 閒置 session 聚合為一行，點擊展開；閒置超過 30 分鐘自動標記過期
- [ ] AC-12: OpenCode adapter — 透過 SSE `/global/event` 偵測 OpenCode 狀態
- [ ] AC-13: Windows 支援 — System Tray 與桌面通知在 Windows 上正常運作（注意：動態視窗建立需使用 async command 避免 deadlock）
- [ ] AC-14: 設定面板 — 使用者可自訂通知偏好、動畫角色、啟動選項
- [ ] AC-15: 浮動小視窗模式 — always-on-top 半透明視窗，可調透明度（30%~100%）、可選大小（小/中）、記住位置
- [ ] AC-16: Linux 支援

## Dependencies

### Rust (Cargo.toml)

| Crate | 用途 |
|-------|------|
| [tauri v2](https://v2.tauri.app/) | 桌面框架，features: `tray-icon` |
| [axum](https://github.com/tokio-rs/axum) | HTTP server（接收 Hook 事件） |
| [reqwest-eventsource](https://docs.rs/reqwest-eventsource/) | SSE client（連線 OpenCode） |
| [tokio](https://tokio.rs/) | 非同步 runtime（Tauri 內建） |
| [serde / serde_json](https://serde.rs/) | JSON 序列化 |
| [tauri-plugin-positioner](https://v2.tauri.app/plugin/positioner/) | Popover 定位到 Tray 下方（需啟用 `tray-icon` feature） |
| [async-trait](https://docs.rs/async-trait/) | Adapter trait 的 async 方法支援 |
| [tauri-plugin-notification](https://v2.tauri.app/plugin/notification/) | macOS/Windows 原生通知 |

### Frontend (package.json)

| Package | 用途 |
|---------|------|
| [React 19](https://react.dev/) | 前端 UI |
| [zustand](https://github.com/pmndrs/zustand) | 前端狀態管理 |
| [lottie-react](https://github.com/LottieFiles/lottie-react) | Lottie 動畫渲染 |
| [@tauri-apps/api](https://www.npmjs.com/package/@tauri-apps/api) | Tauri IPC（invoke / listen） |

### External APIs

- Claude Code Hooks API — agent 狀態偵測（v0.2.0）
- OpenCode SSE API — agent 狀態偵測（Post-MVP）

## Risks

| 風險 | 影響 | 緩解策略 |
|------|------|----------|
| Claude Code Hooks API 變動 | adapter 失效 | 抽象化 adapter 介面，隔離變動影響；Hook 腳本獨立可快速更新 |
| OpenCode SSE 事件格式變動 | adapter 解析錯誤 | 使用官方 SDK 型別定義，版本鎖定 |
| Lottie 動畫檔案過大 | 應用體積膨脹 | 壓縮動畫、限制 keyframe 數量 |
| System Tray API 跨平台差異 | 行為不一致 | MVP 先聚焦 macOS，逐步驗證其他平台 |
| Hook HTTP 轉發失敗 | 漏接 Claude Code 事件 | `async: true` 不阻塞 agent；Code Buddy 未啟動時 Hook 靜默失敗（curl 超時） |
| OpenCode Server 未啟動 | 無法連線 SSE | 定期重連機制（exponential backoff）；UI 顯示「未連線」狀態 |
| `idle` 狀態歧義（OpenCode） | 無法區分「完成」與「等待輸入」 | 結合前一個事件類型判斷（詳見 OpenCode Adapter 的 idle 判斷規則） |
| Tauri `Focused(false)` 已知 Bug | macOS/Linux 多次 `show()` 後 blur 事件可能不觸發 | 追蹤 Tauri GitHub #13633；準備 fallback 方案（clickaway listener 或定時檢查） |
| 多 session 偵測困難 | 無法自動發現 session | Claude Code 透過 `SessionStart` Hook 自動註冊；OpenCode 透過 SSE 事件自動發現 |

---

## Sync Status

### Scope: Project

- [ ] Core Standard: N/A
- [ ] Skill: N/A
- [ ] Command: N/A
- [ ] Translations: N/A
