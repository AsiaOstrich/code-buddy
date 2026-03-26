# [Derived] SPEC-003 — 浮動圓形動畫視窗
# 每個場景對應一個 AC，確保 1:1 追蹤

Feature: SPEC-003 浮動圓形動畫視窗

  Background:
    Given Code Buddy 應用已啟動

  # === AC-1: 啟用/關閉浮動模式 ===

  Scenario: AC-1 — 透過 Tray Menu 啟用浮動模式
    When 使用者右鍵點擊 Tray 圖示
    And 選擇「浮動模式」
    Then 螢幕上出現一個圓形浮動視窗
    And 視窗顯示當前狀態的 Lottie 動畫

  Scenario: AC-1 — 透過 Tray Menu 關閉浮動模式
    Given 浮動視窗已顯示
    When 使用者右鍵點擊 Tray 圖示
    And 選擇「關閉浮動模式」
    Then 圓形浮動視窗消失

  # === AC-2: 圓形、透明、always-on-top ===

  Scenario: AC-2 — 浮動視窗為 always-on-top
    Given 浮動視窗已顯示
    When 使用者切換到其他應用程式
    Then 浮動視窗仍然顯示在所有視窗之上

  # === AC-3: 圓形裁切 ===

  Scenario: AC-3 — 動畫僅在圓形區域內可見
    Given 浮動視窗已顯示
    Then 只有圓形區域內的動畫可見
    And 超出圓形的部分被裁切不顯示
    And 圓形以外的區域完全透明

  # === AC-4: 動畫跟隨狀態切換 ===

  Scenario: AC-4 — 狀態變化時動畫即時切換
    Given 浮動視窗已顯示
    And 當前狀態為 "working"
    When agent 狀態變為 "completed"
    Then 圓形視窗內的動畫切換為 completed 動畫

  # === AC-5: 透明度可調 ===

  Scenario: AC-5 — 預設透明度為 70%
    When 使用者首次啟用浮動模式
    Then 視窗透明度為 70%

  Scenario: AC-5 — 滾輪調整透明度
    Given 浮動視窗已顯示
    When 使用者在視窗上滾動滑鼠滾輪向上
    Then 透明度增加
    And 透明度不超過 100%

  Scenario: AC-5 — 透明度下限為 30%
    Given 浮動視窗已顯示
    And 透明度為 30%
    When 使用者在視窗上滾動滑鼠滾輪向下
    Then 透明度維持 30% 不再降低

  # === AC-6: 預設位置 ===

  Scenario: AC-6 — 首次啟用時位於螢幕右下角
    When 使用者首次啟用浮動模式
    Then 視窗位於螢幕右下角
    And 距離螢幕邊緣 20px

  # === AC-7: 拖拉與記住位置 ===

  Scenario: AC-7 — 拖拉移動視窗
    Given 浮動視窗已顯示
    When 使用者按住視窗並拖拉到螢幕中央
    Then 視窗跟隨滑鼠移動到新位置

  Scenario: AC-7 — 記住位置和透明度
    Given 使用者已拖拉視窗到自訂位置
    And 調整透明度為 50%
    When 關閉並重新啟動 Code Buddy
    And 啟用浮動模式
    Then 視窗出現在上次的位置
    And 透明度為 50%

  # === AC-8: 狀態色邊框 ===

  Scenario Outline: AC-8 — 狀態邊框顏色對應
    Given 浮動視窗已顯示
    When agent 狀態為 "<status>"
    Then 圓形外圍顯示 3px "<color>" 光暈邊框

    Examples:
      | status          | color |
      | idle            | 灰色  |
      | working         | 藍色  |
      | thinking        | 紫色  |
      | waiting_input   | 黃色  |
      | waiting_confirm | 橙色  |
      | completed       | 綠色  |
      | error           | 紅色  |

  # === AC-9: 點擊展開面板 ===

  Scenario: AC-9 — 點擊浮動視窗展開 Popover
    Given 浮動視窗已顯示
    When 使用者點擊圓形視窗
    Then Popover 面板展開
