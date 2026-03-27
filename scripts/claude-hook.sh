#!/bin/bash
# Code Buddy — Claude Code Hook 轉發腳本
# 將 Claude Code hook 事件的 raw JSON 直接轉發到 Code Buddy HTTP server
# 零外部依賴：僅需 curl（Git Bash 自帶）

CODE_BUDDY_PORT=${CODE_BUDDY_PORT:-19199}
CODE_BUDDY_URL="http://localhost:${CODE_BUDDY_PORT}/claude-code/event"

# 讀取 stdin 的 raw JSON，直接轉發（不解析）
STDIN_JSON=$(cat -)

curl -s -X POST "${CODE_BUDDY_URL}" \
  -H "Content-Type: application/json" \
  -d "${STDIN_JSON}" \
  --max-time 2 \
  > /dev/null 2>&1 \
  || true
