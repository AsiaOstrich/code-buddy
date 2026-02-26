#!/usr/bin/env bash
# tauri-dev-stop.sh — 停止背景執行的 tauri dev
# 用法：npm run tauri:stop

set -euo pipefail

PID_FILE="/tmp/code-buddy-dev.pid"

if [ ! -f "$PID_FILE" ]; then
    echo "找不到 PID 檔案，沒有執行中的 Code Buddy dev"
    exit 0
fi

PID=$(cat "$PID_FILE")

if kill -0 "$PID" 2>/dev/null; then
    kill "$PID"
    rm -f "$PID_FILE"
    echo "已停止 Code Buddy dev (PID: $PID)"
else
    rm -f "$PID_FILE"
    echo "程序已不存在，已清理 PID 檔案"
fi
