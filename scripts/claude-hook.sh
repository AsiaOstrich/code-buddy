#!/bin/bash
# Code Buddy — Claude Code Hook 轉發腳本
# v0.2.0 實作時啟用
#
# 使用方式：
# 在 Claude Code 設定中加入此腳本作為 hook
# 腳本會將 hook 事件轉發到 Code Buddy 的 HTTP server
#
# CODE_BUDDY_PORT=${CODE_BUDDY_PORT:-19199}
# CODE_BUDDY_URL="http://localhost:${CODE_BUDDY_PORT}"
#
# # 讀取 stdin 的 hook payload
# PAYLOAD=$(cat -)
#
# # 轉發到 Code Buddy
# curl -s -X POST \
#   "${CODE_BUDDY_URL}/hook/status" \
#   -H "Content-Type: application/json" \
#   -d "${PAYLOAD}" \
#   --max-time 2 \
#   || true  # 靜默失敗，不影響 Claude Code 運作

echo "Code Buddy hook stub — v0.2.0 實作"
