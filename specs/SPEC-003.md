# SPEC-003 Code Buddy — 浮動圓形動畫視窗

> **Status**: Approved
> **Author**: AlbertHsu
> **Created**: 2026-03-25
> **Last Updated**: 2026-03-25
> **Depends on**: SPEC-001 (AC-15)

---

## Overview

新增「浮動模式」：一個 always-on-top 的圓形小視窗，僅顯示當前狀態的 Lottie 動畫（圓形裁切），可調透明度，預設停靠在螢幕右下角。讓開發者不需切換視窗即可持續感知 AI agent 狀態。

## Motivation

- Tray 圓點太小，只能看到顏色，看不到動畫細節
- Popover 面板需要點擊才能看到，不夠「被動」
- 開發者習慣多螢幕，需要一個不干擾的持續監控方式
- 圓形造型與鴕鳥角色配合，像一個「桌面寵物」

## Requirements

### Requirement: 圓形浮動視窗

系統 SHALL 提供一個圓形的 always-on-top 浮動視窗，顯示當前焦點 session 的 Lottie 動畫。

#### Scenario: 啟用浮動模式
- **GIVEN** Code Buddy 正在運行
- **WHEN** 使用者透過右鍵 Tray Menu 選擇「浮動模式」
- **THEN** 螢幕右下角出現一個圓形視窗（預設 80x80px），顯示當前狀態的 Lottie 動畫，圓形外的部分不可見

#### Scenario: 動畫跟隨狀態變化
- **GIVEN** 浮動視窗已顯示
- **WHEN** agent 狀態從 `working` 變為 `completed`
- **THEN** 圓形視窗內的 Lottie 動畫即時切換為 completed 動畫

#### Scenario: 關閉浮動模式
- **GIVEN** 浮動視窗已顯示
- **WHEN** 使用者透過右鍵 Tray Menu 選擇「關閉浮動模式」
- **THEN** 圓形視窗消失

### Requirement: 圓形裁切與透明背景

系統 SHALL 將視窗內容裁切為圓形，視窗背景完全透明，僅圓形動畫可見。

#### Scenario: 圓形遮罩
- **GIVEN** 浮動視窗已顯示
- **WHEN** Lottie 動畫播放中
- **THEN** 只有圓形區域內的動畫可見，超出圓形的部分被裁切不顯示

#### Scenario: 透明背景
- **GIVEN** 浮動視窗已顯示
- **WHEN** 使用者觀察視窗
- **THEN** 圓形以外的區域完全透明，可看到背後的桌面/應用程式

### Requirement: 透明度調整

系統 SHALL 允許使用者調整浮動視窗的透明度。

#### Scenario: 預設透明度
- **GIVEN** 使用者首次啟用浮動模式
- **WHEN** 視窗出現
- **THEN** 透明度為 70%（不完全不透明，降低視覺干擾）

#### Scenario: 調整透明度
- **GIVEN** 浮動視窗已顯示
- **WHEN** 使用者滾動滑鼠滾輪（hover 在視窗上時）
- **THEN** 透明度在 30%~100% 之間調整，即時生效

#### Scenario: 記住透明度設定
- **GIVEN** 使用者調整過透明度
- **WHEN** 下次啟動 Code Buddy 並開啟浮動模式
- **THEN** 使用上次的透明度設定

### Requirement: 位置與拖拉

系統 SHALL 將浮動視窗預設放在螢幕右下角，支援拖拉移動，記住位置。

#### Scenario: 預設位置
- **GIVEN** 使用者首次啟用浮動模式
- **WHEN** 視窗出現
- **THEN** 位於螢幕右下角（距離邊緣 20px）

#### Scenario: 拖拉移動
- **GIVEN** 浮動視窗已顯示
- **WHEN** 使用者按住視窗並拖拉
- **THEN** 視窗跟隨滑鼠移動

#### Scenario: 記住位置
- **GIVEN** 使用者拖拉視窗到新位置
- **WHEN** 下次啟動並開啟浮動模式
- **THEN** 視窗出現在上次的位置

### Requirement: 狀態色邊框

系統 SHALL 在圓形視窗外圍顯示狀態顏色光暈，強化狀態辨識。

#### Scenario: 狀態邊框顏色
- **GIVEN** 浮動視窗已顯示
- **WHEN** agent 狀態為 `working`
- **THEN** 圓形外圍顯示 3px 藍色光暈邊框

#### Scenario: 邊框顏色對應表
- **GIVEN** 浮動視窗已顯示
- **WHEN** 狀態變化
- **THEN** 邊框顏色按以下對應：idle=灰、working=藍、thinking=紫、waiting_input=黃、waiting_confirm=橙、completed=綠、error=紅

### Requirement: 互動

系統 SHALL 支援點擊浮動視窗展開完整面板。

#### Scenario: 點擊展開面板
- **GIVEN** 浮動視窗已顯示
- **WHEN** 使用者點擊圓形視窗
- **THEN** 展開 Popover 面板（同左鍵點擊 Tray 的行為）

## Acceptance Criteria

| AC | 說明 |
|----|------|
| AC-1 | 右鍵 Tray Menu 可啟用/關閉浮動模式 |
| AC-2 | 浮動視窗為圓形，背景透明，always-on-top |
| AC-3 | 僅顯示圓形區域內的 Lottie 動畫，超出部分裁切 |
| AC-4 | 動畫跟隨 agent 狀態即時切換 |
| AC-5 | 透明度可調（30%~100%），預設 70%，滾輪調整 |
| AC-6 | 預設位置為螢幕右下角 |
| AC-7 | 可拖拉移動，記住位置和透明度 |
| AC-8 | 圓形外圍有 3px 狀態顏色光暈邊框 |
| AC-9 | 點擊圓形視窗展開 Popover 面板 |

## Technical Design

### Architecture

```
┌──────────────────────────────────────┐
│            Tauri App                 │
│                                      │
│  ┌─────────────┐  ┌───────────────┐  │
│  │ Main Window │  │ Float Window  │  │
│  │ (Popover)   │  │ (New)         │  │
│  │ 400x600     │  │ 86x86         │  │
│  │ decorations │  │ transparent   │  │
│  │             │  │ no-decorations│  │
│  │             │  │ always-on-top │  │
│  └─────────────┘  └───────────────┘  │
│                                      │
│  ┌─────────────┐                     │
│  │ float.rs    │  ← 視窗管理        │
│  │ - create()  │                     │
│  │ - toggle()  │                     │
│  │ - position()│                     │
│  └─────────────┘                     │
└──────────────────────────────────────┘
```

### Tauri WebviewWindow 設定

```rust
WebviewWindowBuilder::new(app, "float", WebviewUrl::App("float.html".into()))
    .title("")
    .inner_size(86.0, 86.0)         // 80px 動畫 + 6px 邊框
    .decorations(false)              // 無標題列
    .transparent(true)               // 透明背景
    .always_on_top(true)             // 永遠最上層
    .skip_taskbar(true)              // 不顯示在 taskbar
    .resizable(false)                // 不可調整大小
    .position(x, y)                  // 右下角位置
    .build()?;
```

### 前端：float.html

獨立頁面，僅包含圓形裁切的 Lottie 動畫：

```html
<div id="float-container" style="
  width: 80px;
  height: 80px;
  border-radius: 50%;
  overflow: hidden;
  border: 3px solid var(--status-color);
  cursor: grab;
">
  <Lottie animationData={...} loop />
</div>
```

### 拖拉實作

使用 Tauri 的 `startDragging()` API：
- `mousedown` → `appWindow.startDragging()`
- 拖拉結束後儲存位置到 localStorage

### 透明度調整

```rust
// Rust 端
window.set_opacity(opacity)?;  // 0.3 ~ 1.0

// 前端 wheel 事件
window.addEventListener('wheel', (e) => {
  opacity += e.deltaY > 0 ? -0.05 : 0.05;
  invoke('set_float_opacity', { opacity });
});
```

### 持久化設定

```json
// 儲存在 app_data_dir/float-settings.json
{
  "enabled": true,
  "x": 1820,
  "y": 980,
  "opacity": 0.7
}
```

### 新增檔案

| 檔案 | 說明 |
|------|------|
| `src-tauri/src/float.rs` | 浮動視窗 Rust 邏輯 |
| `src/float.html` | 浮動視窗獨立 HTML 入口 |
| `src/FloatApp.tsx` | 浮動視窗 React 元件 |

### Tray Menu 修改

```rust
// tray.rs — 新增選項
let float_toggle = MenuItem::with_id(app, "float", "浮動模式", true, None::<&str>)?;
```

## Risks

| 風險 | 影響 | 緩解 |
|------|------|------|
| 圓形透明視窗在 Linux 可能不支援 | 外觀異常 | 先 macOS，Linux 降級為方形 |
| 滑鼠穿透問題 | 使用者抱怨擋住點擊 | 視窗很小（80px），影響有限 |
| 多螢幕位置記憶 | 換螢幕後位置可能在螢幕外 | 啟動時檢查位置是否在螢幕範圍內 |

## Out of Scope

- 多種尺寸（小/中/大）— 僅做 80x80 圓形
- 浮動視窗內顯示文字 — 僅動畫
- 滑鼠穿透模式 — 未來考慮
- 自訂快捷鍵切換浮動模式
