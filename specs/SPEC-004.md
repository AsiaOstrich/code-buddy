# SPEC-004 Code Buddy — 消除 jq 依賴與 Server 端 Raw JSON 解析

> **Status**: Draft
> **Author**: AlbertHsu
> **Created**: 2026-03-27
> **Last Updated**: 2026-03-27
> **Depends on**: SPEC-001 (AC-5, AC-6)

---

## Overview

將 hook 腳本簡化為「直接轉發 raw JSON」，移除對 `jq` 的依賴。所有 JSON 解析邏輯搬移至 Rust server 端，使 Code Buddy 在 Windows（Git Bash）環境下零外部依賴即可運作。

## Motivation

- Windows 環境普遍未預裝 `jq`，導致 `claude-hook.sh` 無法正常解析 hook payload
- 現行腳本對 `jq` 的 6 次呼叫造成不必要的複雜度
- Rust server 端已具備 `serde_json` 解析能力，重複解析 JSON 是浪費
- 只需要 `curl`（Git Bash 自帶）即可完成轉發

## Requirements

### Requirement 1: Hook 腳本零依賴轉發

系統 SHALL 提供一個僅依賴 `curl` 的 hook 腳本，將 Claude Code 的 stdin JSON 原封不動轉發至 Code Buddy HTTP server。

#### Scenario: 正常轉發 hook 事件
- **GIVEN** Claude Code 觸發一個 hook 事件（例如 `SessionStart`）
- **WHEN** hook 腳本從 stdin 接收到 JSON payload
- **THEN** 腳本將整個 JSON 作為 POST body 轉發至 `http://localhost:${CODE_BUDDY_PORT}/claude-code/event`
- **AND** 不依賴 `jq` 或其他外部 JSON 解析工具

#### Scenario: Server 未運行時靜默失敗
- **GIVEN** Code Buddy HTTP server 未啟動
- **WHEN** hook 腳本嘗試轉發
- **THEN** 腳本在 2 秒內逾時並靜默退出（exit code 0）
- **AND** 不阻塞 Claude Code 的正常運作

#### Scenario: 環境變數自訂 Port
- **GIVEN** 環境變數 `CODE_BUDDY_PORT` 設為 `29199`
- **WHEN** hook 腳本執行
- **THEN** 轉發目標為 `http://localhost:29199/claude-code/event`

### Requirement 2: Server 端相容 Raw JSON 格式

系統 SHALL 在 HTTP server 端接受 Claude Code 原始 hook JSON 格式，並正確解析所有必要欄位。

#### Scenario: 接收 Claude Code 原始 SessionStart payload
- **GIVEN** server 收到 Claude Code 原始格式的 JSON：
  ```json
  {
    "hook_event_name": "SessionStart",
    "session_id": "abc-123",
    "cwd": "/Users/dev/my-project"
  }
  ```
- **WHEN** server 處理此 payload
- **THEN** 正確識別 `hook_event_name` 為 `SessionStart`
- **AND** 正確識別 `session_id` 為 `abc-123`
- **AND** 從 `cwd` 欄位提取 `project_path`（因原始格式使用 `cwd` 而非 `project_path`）

#### Scenario: 接收含 notification_type 的 Notification payload
- **GIVEN** server 收到：
  ```json
  {
    "hook_event_name": "Notification",
    "session_id": "abc-123",
    "cwd": "/Users/dev/my-project",
    "notification_type": "permission_prompt"
  }
  ```
- **WHEN** server 處理此 payload
- **THEN** 狀態正確轉換為 `WaitingConfirm`

#### Scenario: 向後相容現有 hook 腳本格式
- **GIVEN** server 收到現有封裝格式的 JSON（含 `project_path` 和 `raw` 欄位）
- **WHEN** server 處理此 payload
- **THEN** 行為與改動前完全一致

### Requirement 3: 完整 hook 事件對應

系統 SHALL 在接收 raw JSON 時，對所有 7 種 hook 事件產生正確的狀態轉換。

#### Scenario: 各事件對應的狀態
| Hook Event | 預期狀態 |
|---|---|
| `SessionStart` | `Idle`（建立新 session） |
| `UserPromptSubmit` | `Thinking` |
| `PostToolUse` | `Working` |
| `PostToolUseFailure` | `Working`（< 3 次）/ `Error`（>= 3 次） |
| `Stop` | `Completed` |
| `Notification` (idle_prompt) | `WaitingInput` |
| `Notification` (permission_prompt) | `WaitingConfirm` |
| `SessionEnd` | `Idle`（移除 session） |

---

## Acceptance Criteria

- [ ] **AC-1**: hook 腳本（`scripts/claude-hook.sh`）不包含任何 `jq` 呼叫
- [ ] **AC-2**: hook 腳本僅依賴 `curl`，在僅有 Git Bash 的 Windows 環境可正常執行
- [ ] **AC-3**: server 未運行時，hook 腳本在 2 秒內靜默退出且 exit code 為 0
- [ ] **AC-4**: server 端接受 Claude Code 原始 JSON 格式（使用 `cwd` 欄位）
- [ ] **AC-5**: server 端同時相容現有封裝格式（使用 `project_path` 欄位）
- [ ] **AC-6**: 所有 7 種 hook 事件的狀態轉換行為不變（與 SPEC-001 AC-5/AC-6 一致）
- [ ] **AC-7**: 現有 Rust 單元測試全數通過
- [ ] **AC-8**: 新增測試覆蓋 raw JSON 格式（`cwd` fallback）的解析

## Technical Design

### 1. Hook 腳本簡化（`scripts/claude-hook.sh`）

**Before** (39 行，依賴 jq):
```bash
STDIN_JSON=$(cat -)
HOOK_EVENT_NAME=$(echo "$STDIN_JSON" | jq -r '.hook_event_name // "unknown"')
# ... 6 次 jq 呼叫 ...
curl -s -X POST "${CODE_BUDDY_URL}" -d "${COMBINED_PAYLOAD}" ...
```

**After** (~10 行，僅依賴 curl):
```bash
#!/bin/bash
CODE_BUDDY_PORT=${CODE_BUDDY_PORT:-19199}
CODE_BUDDY_URL="http://localhost:${CODE_BUDDY_PORT}/claude-code/event"

# 讀取 stdin 的 raw JSON，直接轉發
STDIN_JSON=$(cat -)

curl -s -X POST "${CODE_BUDDY_URL}" \
  -H "Content-Type: application/json" \
  -d "${STDIN_JSON}" \
  --max-time 2 \
  > /dev/null 2>&1 \
  || true
```

### 2. Server 端 Payload 相容（`src-tauri/src/adapters/claude_code.rs`）

修改 `HookPayload` struct，新增 `cwd` 欄位做為 `project_path` 的 fallback：

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookPayload {
    pub hook_event_name: String,
    pub session_id: String,
    #[serde(default)]
    pub project_path: String,
    #[serde(default)]
    pub cwd: Option<String>,          // ← 新增：Claude Code 原始欄位
    #[serde(default)]
    pub notification_type: Option<String>,
    #[serde(default)]
    pub raw: Option<serde_json::Value>,
}
```

在 `process_hook_event()` 中解析 project path：

```rust
let project_path = if payload.project_path.is_empty() {
    payload.cwd.as_deref().unwrap_or("")
} else {
    &payload.project_path
};
```

### 3. 影響範圍

| 檔案 | 變更類型 | 說明 |
|------|----------|------|
| `scripts/claude-hook.sh` | **重寫** | 移除 jq，簡化為 raw 轉發 |
| `src-tauri/src/adapters/claude_code.rs` | **修改** | HookPayload 新增 `cwd`，project_path fallback 邏輯 |
| `src-tauri/src/server.rs` | **無變更** | 已透過 `Json<HookPayload>` 自動反序列化 |

## Test Plan

- [ ] `claude_code.rs` 單元測試：raw JSON 格式（含 `cwd`，無 `project_path`）正確解析
- [ ] `claude_code.rs` 單元測試：同時有 `project_path` 和 `cwd` 時，優先使用 `project_path`
- [ ] `claude_code.rs` 單元測試：兩者皆空時，使用空字串（不 panic）
- [ ] `server.rs` 整合測試：POST raw JSON 格式到 `/claude-code/event` 回傳 200
- [ ] 手動測試：在無 `jq` 的 Windows Git Bash 中執行 hook 腳本，確認事件到達 server
- [ ] 手動測試：server 未啟動時，hook 腳本 2 秒內退出

---

> **規格文件已建立。建議下一步：**
> - 審查此規格後標記為 `Approved`
> - 執行 `/tdd-assistant` 從 AC 推導測試骨架
> - 或直接開始實作（範圍小，約 3 個檔案修改）
