# [Source] SPEC-001 — v0.3.0 Panel（面板）
# 完整 MVP 體驗

Feature: v0.3.0 Popover 面板
  作為開發者
  我希望點擊 Tray 圖示能展開詳細面板
  以便我查看 session 的動畫狀態和詳細資訊

  Background:
    Given Code Buddy 應用已啟動
    And 至少有一個 session 正在追蹤中

  # [Derived] AC-8: 左鍵點擊 Tray 圖示展開 Popover 面板，點擊外部或 ESC 關閉
  Scenario: AC-8 — 左鍵點擊展開 Popover 面板
    When 左鍵點擊 System Tray 圖示
    Then 應展開 Popover 面板
    And 面板應定位於 Tray 圖示下方

  Scenario: AC-8 — 再次左鍵點擊關閉 Popover 面板
    Given Popover 面板已展開
    When 左鍵點擊 System Tray 圖示
    Then Popover 面板應關閉

  Scenario: AC-8 — 點擊面板外部關閉
    Given Popover 面板已展開
    When 點擊面板外部區域
    Then Popover 面板應關閉

  Scenario: AC-8 — ESC 鍵關閉面板
    Given Popover 面板已展開
    When 按下 ESC 鍵
    Then Popover 面板應關閉

  # [Derived] AC-9: 面板顯示當前 session 的狀態動畫 + 狀態文字 + 持續時間
  Scenario: AC-9 — 面板顯示焦點 session 的 Lottie 動畫
    Given 焦點 session 的狀態為 "working"
    When Popover 面板展開
    Then 面板應顯示 "working" 對應的 Lottie 動畫
    And 動畫應為循環播放

  Scenario: AC-9 — 面板顯示狀態文字和持續時間
    Given 焦點 session "my-project" 的狀態為 "working"
    And session 已持續 "3m 42s"
    When Popover 面板展開
    Then 面板應顯示狀態文字 "工作中"
    And 面板應顯示持續時間 "3m 42s"
    And 面板應顯示專案名稱 "my-project"

  Scenario: AC-9 — completed/error 動畫播放一次後靜止
    Given 焦點 session 的狀態為 "completed"
    When Popover 面板展開
    Then 面板應顯示 "completed" 對應的 Lottie 動畫
    And 動畫應播放一次後靜止

  Scenario: AC-9 — 焦點自動跟隨最近狀態變化的 session
    Given 有 session "project-a" 狀態為 "idle"
    And 有 session "project-b" 狀態為 "idle"
    And 無釘選的 session
    When session "project-b" 狀態變為 "working"
    Then 焦點應自動切換至 "project-b"

  # [Derived] AC-10: 右鍵 Context Menu 擴充（靜音通知 30 分鐘、設定）
  Scenario: AC-10 — 右鍵 Context Menu 包含靜音選項
    When 右鍵點擊 System Tray 圖示
    Then Context Menu 應包含「靜音通知 30 分鐘」選項
    And Context Menu 應包含「設定...」選項

  Scenario: AC-10 — 靜音通知 30 分鐘
    When 右鍵點擊 System Tray 圖示
    And 選擇「靜音通知 30 分鐘」
    Then 通知應被靜音
    And 靜音應在 30 分鐘後自動解除

  Scenario: AC-10 — 靜音期間不推送通知
    Given 通知已被靜音
    When session 狀態變為 "waiting_input"
    Then 不應推送桌面通知
    But System Tray 圖示仍應正常切換
