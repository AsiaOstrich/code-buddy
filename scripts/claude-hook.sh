#!/bin/bash
# Code Buddy — Claude Code Hook 轉發腳本
# 將 Claude Code hook 事件轉發到 Code Buddy HTTP server

CODE_BUDDY_PORT=${CODE_BUDDY_PORT:-19199}
CODE_BUDDY_URL="http://localhost:${CODE_BUDDY_PORT}/claude-code/event"

# 讀取 stdin 的 hook JSON payload
STDIN_JSON=$(cat -)

# 從 JSON 解析欄位
HOOK_EVENT_NAME=$(echo "$STDIN_JSON" | jq -r '.hook_event_name // "unknown"')
SESSION_ID=$(echo "$STDIN_JSON" | jq -r '.session_id // "unknown"')
PROJECT_PATH="${CLAUDE_PROJECT_DIR:-$(echo "$STDIN_JSON" | jq -r '.cwd // ""')}"
NOTIFICATION_TYPE=$(echo "$STDIN_JSON" | jq -r '.notification_type // empty' 2>/dev/null)

# 組合 payload
COMBINED_PAYLOAD=$(jq -n \
  --arg hook "$HOOK_EVENT_NAME" \
  --arg sid "$SESSION_ID" \
  --arg path "$PROJECT_PATH" \
  --arg ntype "${NOTIFICATION_TYPE:-}" \
  --argjson raw "$STDIN_JSON" \
  '{
    hook_event_name: $hook,
    session_id: $sid,
    project_path: $path,
    notification_type: (if $ntype == "" then null else $ntype end),
    raw: $raw
  }')

# 轉發到 Code Buddy（靜默失敗，不阻塞 Claude Code）
curl -s -X POST \
  "${CODE_BUDDY_URL}" \
  -H "Content-Type: application/json" \
  -d "${COMBINED_PAYLOAD}" \
  --max-time 2 \
  > /dev/null 2>&1 \
  || true
