# [Source] SPEC-004 — 消除 jq 依賴與 Server 端 Raw JSON 解析
# 驗證 hook 腳本零依賴轉發 + server 端相容 raw JSON 格式

Feature: SPEC-004 Raw JSON Hook 轉發
  作為 Windows 開發者
  我希望 hook 腳本不依賴 jq 即可運作
  以便我在僅有 Git Bash 的環境下也能偵測 Claude Code session

  Background:
    Given Code Buddy HTTP server 正在 localhost:19199 運行

  # ============================================================
  # [Derived] AC-1: hook 腳本不包含任何 jq 呼叫
  # ============================================================

  Scenario: AC-1 — hook 腳本不依賴 jq
    Given hook 腳本 "scripts/claude-hook.sh" 存在
    When 檢查腳本內容
    Then 腳本中不應包含 "jq" 字串
    And 腳本中不應包含 "python" 或 "node" 呼叫

  # ============================================================
  # [Derived] AC-2: hook 腳本僅依賴 curl
  # ============================================================

  Scenario: AC-2 — hook 腳本僅使用 curl 轉發
    Given hook 腳本 "scripts/claude-hook.sh" 存在
    When 檢查腳本中的外部指令呼叫
    Then 唯一的外部指令應為 "curl"

  # ============================================================
  # [Derived] AC-3: server 未運行時靜默失敗
  # ============================================================

  Scenario: AC-3 — server 未啟動時 hook 腳本靜默退出
    Given Code Buddy HTTP server 未啟動
    And 環境中沒有 jq
    When 執行 hook 腳本，stdin 輸入為：
      """
      {
        "hook_event_name": "SessionStart",
        "session_id": "test-001",
        "cwd": "/Users/dev/my-project"
      }
      """
    Then 腳本應在 2 秒內退出
    And exit code 應為 0

  # ============================================================
  # [Derived] AC-4: server 接受 Claude Code 原始 JSON 格式（cwd 欄位）
  # ============================================================

  Scenario: AC-4 — server 接受含 cwd 的原始 SessionStart payload
    When 發送 POST 至 "/claude-code/event"，body 為：
      """
      {
        "hook_event_name": "SessionStart",
        "session_id": "raw-001",
        "cwd": "/Users/dev/my-project"
      }
      """
    Then HTTP 回應狀態碼應為 200
    And 回應 JSON 的 "ok" 應為 true
    And session "raw-001" 的 project_path 應為 "/Users/dev/my-project"
    And session "raw-001" 的 project_name 應為 "my-project"

  Scenario: AC-4 — server 接受含 cwd 的原始 Notification payload
    Given 已透過 raw JSON 註冊 session "raw-002"
    When 發送 POST 至 "/claude-code/event"，body 為：
      """
      {
        "hook_event_name": "Notification",
        "session_id": "raw-002",
        "cwd": "/Users/dev/my-project",
        "notification_type": "permission_prompt"
      }
      """
    Then HTTP 回應狀態碼應為 200
    And session "raw-002" 的狀態應為 "waiting_confirm"

  Scenario: AC-4 — server 從 cwd 正確提取 project_name
    When 發送 POST 至 "/claude-code/event"，body 為：
      """
      {
        "hook_event_name": "SessionStart",
        "session_id": "raw-003",
        "cwd": "C:\\Users\\dev\\Documents\\my-app"
      }
      """
    Then session "raw-003" 的 project_name 應為 "my-app"

  # ============================================================
  # [Derived] AC-5: 向後相容現有封裝格式
  # ============================================================

  Scenario: AC-5 — server 相容含 project_path 的舊格式
    When 發送 POST 至 "/claude-code/event"，body 為：
      """
      {
        "hook_event_name": "SessionStart",
        "session_id": "compat-001",
        "project_path": "/Users/dev/old-format-project",
        "notification_type": null,
        "raw": {"hook_event_name": "SessionStart"}
      }
      """
    Then HTTP 回應狀態碼應為 200
    And session "compat-001" 的 project_path 應為 "/Users/dev/old-format-project"

  Scenario: AC-5 — 同時有 project_path 和 cwd 時優先使用 project_path
    When 發送 POST 至 "/claude-code/event"，body 為：
      """
      {
        "hook_event_name": "SessionStart",
        "session_id": "priority-001",
        "project_path": "/from/project_path",
        "cwd": "/from/cwd"
      }
      """
    Then session "priority-001" 的 project_path 應為 "/from/project_path"

  # ============================================================
  # [Derived] AC-6: 所有 7 種 hook 事件的狀態轉換不變
  # ============================================================

  Scenario Outline: AC-6 — raw JSON 格式下 <event> 事件產生 <status> 狀態
    Given 已透過 raw JSON 註冊 session "state-001"
    When 發送 raw JSON hook 事件 "<event>" 給 session "state-001"
    Then session "state-001" 的狀態應為 "<status>"

    Examples:
      | event             | status          |
      | UserPromptSubmit  | thinking        |
      | PostToolUse       | working         |
      | Stop              | completed       |

  Scenario: AC-6 — raw JSON 格式下 SessionEnd 移除 session
    Given 已透過 raw JSON 註冊 session "state-002"
    When 發送 raw JSON hook 事件 "SessionEnd" 給 session "state-002"
    Then session "state-002" 應從列表中移除

  Scenario: AC-6 — raw JSON 格式下 PostToolUseFailure 防抖機制不變
    Given 已透過 raw JSON 註冊 session "state-003"
    When 發送 1 次 raw JSON "PostToolUseFailure" 事件
    Then session "state-003" 的狀態應為 "working"
    When 再發送 2 次 raw JSON "PostToolUseFailure" 事件
    Then session "state-003" 的狀態應為 "error"

  # ============================================================
  # [Derived] AC-8: 新增測試覆蓋 cwd fallback 解析
  # ============================================================

  Scenario: AC-8 — 兩欄位皆空時使用空字串
    When 發送 POST 至 "/claude-code/event"，body 為：
      """
      {
        "hook_event_name": "PostToolUse",
        "session_id": "empty-path-001"
      }
      """
    Then HTTP 回應狀態碼應為 200
    And session "empty-path-001" 的 project_path 應為 ""
