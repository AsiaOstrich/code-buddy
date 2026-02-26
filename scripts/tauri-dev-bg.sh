#!/usr/bin/env bash
# tauri-dev-bg.sh — 在背景執行 tauri dev，不佔用終端機
# 用法：npm run tauri:bg

set -euo pipefail

LOG_FILE="/tmp/code-buddy-dev.log"
PID_FILE="/tmp/code-buddy-dev.pid"

# 如果已有執行中的 instance，先提示
if [ -f "$PID_FILE" ]; then
    OLD_PID=$(cat "$PID_FILE")
    if kill -0 "$OLD_PID" 2>/dev/null; then
        echo "Code Buddy dev 已在執行中 (PID: $OLD_PID)"
        echo "如需重啟，請先執行: kill $OLD_PID"
        exit 1
    fi
fi

echo "啟動 Code Buddy dev（背景模式）..."
echo "Log: $LOG_FILE"

nohup npx tauri dev > "$LOG_FILE" 2>&1 &
DEV_PID=$!
echo "$DEV_PID" > "$PID_FILE"

echo "PID: $DEV_PID"
echo "停止方式: kill $DEV_PID"
