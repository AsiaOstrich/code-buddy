# [Source] SPEC-001 — v0.2.0 Detection（偵測）
# 驗證核心價值「開發者可以被動感知 agent 狀態」

Feature: v0.2.0 Claude Code 狀態偵測
  作為開發者
  我希望 Code Buddy 自動偵測 Claude Code 的狀態
  以便我無需切換視窗即可感知 agent 動態

  Background:
    Given Code Buddy 應用已啟動
    And HTTP server 正在 localhost:3456 運行

  # [Derived] AC-5: 可偵測本機至少一個 Claude Code session 的狀態（HTTP server + Hook 整合）
  Scenario: AC-5 — 接收 Claude Code SessionStart Hook 事件
    When Claude Code 發送 SessionStart Hook 事件至 "/claude-code/event"
      """
      {
        "session_id": "test-session-001",
        "cwd": "/Users/user/my-project",
        "hook_event_name": "SessionStart"
      }
      """
    Then Code Buddy 應註冊新 session "test-session-001"
    And session 的 agent_type 應為 "claude_code"
    And session 的 project_name 應為 "my-project"

  Scenario: AC-5 — HTTP server 健康檢查
    When 發送 GET 請求至 "/health"
    Then 應回傳 HTTP 200

  Scenario: AC-5 — 查詢 session 列表
    Given 已有註冊的 session "test-session-001"
    When 發送 GET 請求至 "/sessions"
    Then 應回傳包含 "test-session-001" 的 session 列表

  # [Derived] AC-6: System Tray 圖示根據真實 Hook 事件即時切換
  Scenario: AC-6 — UserPromptSubmit 事件觸發 thinking 狀態
    Given 已有註冊的 session "test-session-001"
    When Claude Code 發送 UserPromptSubmit Hook 事件
    Then session "test-session-001" 的狀態應為 "thinking"
    And System Tray 圖示應切換為 "thinking"

  Scenario: AC-6 — PostToolUse 事件觸發 working 狀態
    Given 已有註冊的 session "test-session-001"
    And session 目前狀態為 "thinking"
    When Claude Code 發送 PostToolUse Hook 事件
    Then session "test-session-001" 的狀態應為 "working"
    And System Tray 圖示應切換為 "working"

  Scenario: AC-6 — Stop 事件觸發 completed 狀態
    Given 已有註冊的 session "test-session-001"
    And session 目前狀態為 "working"
    When Claude Code 發送 Stop Hook 事件
    Then session "test-session-001" 的狀態應為 "completed"
    And System Tray 圖示應切換為 "completed"

  Scenario: AC-6 — Notification (idle_prompt) 觸發 waiting_input 狀態
    Given 已有註冊的 session "test-session-001"
    When Claude Code 發送 Notification Hook 事件，notification_type 為 "idle_prompt"
    Then session "test-session-001" 的狀態應為 "waiting_input"
    And System Tray 圖示應切換為 "waiting_input"

  Scenario: AC-6 — Notification (permission_prompt) 觸發 waiting_confirm 狀態
    Given 已有註冊的 session "test-session-001"
    When Claude Code 發送 Notification Hook 事件，notification_type 為 "permission_prompt"
    Then session "test-session-001" 的狀態應為 "waiting_confirm"
    And System Tray 圖示應切換為 "waiting_confirm"

  Scenario: AC-6 — thinking 狀態推導（5 秒無新事件）
    Given 已有註冊的 session "test-session-001"
    And session 目前狀態為 "working"
    When 超過 5 秒未收到新 Hook 事件
    Then session "test-session-001" 的狀態應為 "thinking"

  Scenario: AC-6 — 狀態防抖：單次 PostToolUseFailure 不觸發 error
    Given 已有註冊的 session "test-session-001"
    And session 目前狀態為 "working"
    When Claude Code 發送 1 次 PostToolUseFailure Hook 事件
    Then session "test-session-001" 的狀態應維持 "working"
    And session 應附加 lastWarning 訊息

  Scenario: AC-6 — 狀態防抖：連續 3 次失敗觸發 error
    Given 已有註冊的 session "test-session-001"
    And session 目前狀態為 "working"
    When Claude Code 在 60 秒內發送 3 次 PostToolUseFailure Hook 事件
    Then session "test-session-001" 的狀態應為 "error"

  Scenario: AC-6 — SessionEnd 事件移除 session
    Given 已有註冊的 session "test-session-001"
    When Claude Code 發送 SessionEnd Hook 事件
    Then session "test-session-001" 應從列表中移除

  # [Derived] AC-7: 狀態變為 completed/waiting_input/waiting_confirm/error 時推送桌面通知
  Scenario Outline: AC-7 — <status> 狀態觸發桌面通知
    Given 已有註冊的 session "test-session-001"
    And 通知未被靜音
    When session 狀態變為 "<status>"
    Then 應推送 macOS 桌面通知
    And 通知級別應為 "<level>"

    Examples:
      | status          | level |
      | waiting_input   | 關鍵  |
      | waiting_confirm | 關鍵  |
      | completed       | 重要  |
      | error           | 重要  |

  Scenario: AC-7 — working/thinking 狀態不推送桌面通知
    Given 已有註冊的 session "test-session-001"
    When session 狀態變為 "working"
    Then 不應推送桌面通知

  Scenario: AC-7 — idle 狀態不推送桌面通知
    Given 已有註冊的 session "test-session-001"
    When session 狀態變為 "idle"
    Then 不應推送桌面通知
