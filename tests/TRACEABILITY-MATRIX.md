# SPEC-001 追蹤矩陣：AC ↔ BDD ↔ TDD

> [Source] 自動產生自 SPEC-001.md
> [Generated] 確保每個 AC 都有對應的 BDD 場景和 TDD 測試

## v0.1.0 — Skeleton（骨架）

| AC | 說明 | BDD 場景 | TDD 測試 | 標籤 |
|----|------|----------|----------|------|
| AC-1 | System Tray 常駐，不佔 Dock | `v0.1.0-skeleton.feature` → "AC-1 — System Tray 常駐且不佔用 Dock" | [TODO] 需 E2E 測試框架 | [TODO] |
| AC-2 | 7 種狀態圖示切換 | `v0.1.0-skeleton.feature` → "AC-2 — System Tray 圖示切換至 \<status\> 狀態" (7 examples) | `state_test.rs` → priority 順序、序列化格式 | [Derived] |
| AC-3 | 右鍵 Context Menu | `v0.1.0-skeleton.feature` → "AC-3 — 右鍵顯示 Context Menu" + "AC-3 — 點擊退出選項關閉應用" | [TODO] 需 E2E 測試框架 | [TODO] |
| AC-4 | 應用 < 20MB，記憶體 < 100MB | `v0.1.0-skeleton.feature` → "AC-4 — 應用大小限制" + "AC-4 — 記憶體使用限制" | [TODO] CI pipeline 中驗證 | [TODO] |

## v0.2.0 — Detection（偵測）

| AC | 說明 | BDD 場景 | TDD 測試 | 標籤 |
|----|------|----------|----------|------|
| AC-5 | Claude Code session 偵測 | `v0.2.0-detection.feature` → "AC-5 — 接收 SessionStart" + "AC-5 — 健康檢查" + "AC-5 — 查詢 session 列表" | `claude_code_adapter_test.rs` → session 註冊/移除/容錯/project_name 提取；`server_test.rs` → HTTP 端點 | [Derived] |
| AC-6 | Tray 圖示即時切換 | `v0.2.0-detection.feature` → 9 個場景（各狀態轉換 + 防抖 + thinking 推導） | `claude_code_adapter_test.rs` → 所有狀態轉換、防抖計數、失敗重置；`state_test.rs` → aggregate/effective status | [Derived] |
| AC-7 | 桌面通知推送 | `v0.2.0-detection.feature` → "AC-7 — \<status\> 狀態觸發桌面通知" (4 examples) + 2 個不通知場景 | `notification_test.rs` → should_notify 判斷、通知文案、防重複 | [Derived] |

## v0.3.0 — Panel（面板）

| AC | 說明 | BDD 場景 | TDD 測試 | 標籤 |
|----|------|----------|----------|------|
| AC-8 | Popover 面板 toggle | `v0.3.0-panel.feature` → 4 個場景（開/關/外部點擊/ESC） | `store.test.ts` → 初始狀態、pinSession | [Derived] |
| AC-9 | 面板動畫 + 狀態資訊 | `v0.3.0-panel.feature` → 4 個場景（動畫/文字/一次性/焦點跟隨） | `BuddyAnimation.test.tsx` → 動畫載入/循環/切換；`SessionList.test.tsx` → 列表顯示/排序；`store.test.ts` → state-changed 更新 | [Derived] |
| AC-10 | Context Menu 擴充 | `v0.3.0-panel.feature` → 3 個場景（靜音選項/30分鐘靜音/靜音期間行為） | `notification_test.rs` → 靜音機制 | [Derived] |

## 統計

| 指標 | 數量 |
|------|------|
| MVP AC 總數 | 10 |
| BDD 場景總數 | 30 |
| TDD 測試檔案 | 7（4 Rust + 3 React） |
| TDD 測試案例 | 38 |
| 完整覆蓋的 AC | 10/10 (100%) |
| [TODO] 待實作 | 通知私有函式測試、HTTP 整合測試、E2E 測試 |

## 檔案索引

### BDD 場景（`.feature`）

| 檔案 | 涵蓋 AC |
|------|---------|
| `tests/bdd/features/v0.1.0-skeleton.feature` | AC-1, AC-2, AC-3, AC-4 |
| `tests/bdd/features/v0.2.0-detection.feature` | AC-5, AC-6, AC-7 |
| `tests/bdd/features/v0.3.0-panel.feature` | AC-8, AC-9, AC-10 |

### TDD 骨架（Rust）

| 檔案 | 涵蓋 AC | 測試目標 |
|------|---------|----------|
| `tests/tdd/rust/state_test.rs` | AC-2, AC-6, AC-9 | 核心型別、優先級、聚合邏輯 |
| `tests/tdd/rust/claude_code_adapter_test.rs` | AC-5, AC-6 | Hook 事件處理、狀態轉換 |
| `tests/tdd/rust/notification_test.rs` | AC-7, AC-10 | 通知觸發、文案、靜音 |
| `tests/tdd/rust/server_test.rs` | AC-5 | HTTP 端點 |

### TDD 骨架（React）

| 檔案 | 涵蓋 AC | 測試目標 |
|------|---------|----------|
| `tests/tdd/react/store.test.ts` | AC-8, AC-9 | Zustand store |
| `tests/tdd/react/SessionList.test.tsx` | AC-9 | Session 列表元件 |
| `tests/tdd/react/BuddyAnimation.test.tsx` | AC-9 | Lottie 動畫元件 |
