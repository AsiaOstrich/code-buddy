# [Source] SPEC-001 — v0.1.0 Skeleton（骨架）
# 驗證 Tauri v2 + System Tray 的技術可行性

Feature: v0.1.0 System Tray 骨架應用
  作為開發者
  我希望 Code Buddy 以 System Tray 常駐應用執行
  以便我在不佔用 Dock 的情況下使用它

  # [Derived] AC-1: 應用程式可在 macOS 上以 System Tray 常駐，不佔用 Dock
  Scenario: AC-1 — System Tray 常駐且不佔用 Dock
    Given Code Buddy 應用已啟動
    When 應用進入 System Tray 模式
    Then System Tray 應顯示鴕鳥圖示
    And 應用不應出現在 macOS Dock
    And 應用不應出現在 Cmd+Tab 切換列表中

  # [Derived] AC-2: System Tray 圖示可手動切換狀態（靜態 PNG），驗證 7 種狀態圖示顯示正確
  Scenario Outline: AC-2 — System Tray 圖示切換至 <status> 狀態
    Given Code Buddy 應用已啟動
    And System Tray 顯示預設圖示
    When 狀態切換為 "<status>"
    Then System Tray 應顯示 "<status>" 對應的 PNG 圖示
    And 圖示顏色應為 "<color>"

    Examples:
      | status          | color |
      | working         | 藍色  |
      | thinking        | 紫色  |
      | waiting_input   | 黃色  |
      | completed       | 綠色  |
      | error           | 紅色  |
      | idle            | 灰色  |
      | waiting_confirm | 橙色  |

  # [Derived] AC-3: 右鍵點擊 Tray 圖示顯示 Context Menu（退出、關於）
  Scenario: AC-3 — 右鍵顯示 Context Menu
    Given Code Buddy 應用已啟動
    When 右鍵點擊 System Tray 圖示
    Then 應顯示 Context Menu
    And Context Menu 應包含「關於 Code Buddy」選項
    And Context Menu 應包含「退出」選項

  Scenario: AC-3 — 點擊退出選項關閉應用
    Given Code Buddy 應用已啟動
    And Context Menu 已展開
    When 點擊「退出」選項
    Then 應用應完全關閉
    And System Tray 圖示應消失

  # [Derived] AC-4: 應用大小 < 20MB，記憶體使用 < 100MB
  Scenario: AC-4 — 應用大小限制
    Given Code Buddy 已完成建置
    When 檢查應用程式包大小
    Then 應用大小應小於 20MB

  Scenario: AC-4 — 記憶體使用限制
    Given Code Buddy 應用已啟動
    And 應用已運行至少 30 秒
    When 檢查應用記憶體使用量
    Then 記憶體使用應小於 100MB
