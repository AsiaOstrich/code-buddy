// [Source] 專案健康報告 — 行動 2
// 測試目標：src/App.tsx — 目前唯一有實作的前端元件

import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import App from "../../../src/App";

// Mock Tauri API
const mockInvoke = vi.fn();
const mockListen = vi.fn();

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => mockInvoke(...args),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: (...args: unknown[]) => mockListen(...args),
}));

describe("App", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockInvoke.mockResolvedValue([]);
    mockListen.mockResolvedValue(vi.fn()); // unlisten function
  });

  // === 基本渲染 ===

  it("顯示 Code Buddy Dev Panel 標題", () => {
    render(<App />);
    expect(screen.getByText("Code Buddy Dev Panel")).toBeInTheDocument();
  });

  it("初始有效狀態為 idle", () => {
    render(<App />);
    expect(screen.getByText("idle")).toBeInTheDocument();
  });

  it("顯示 7 個狀態切換按鈕", () => {
    render(<App />);
    const buttons = screen.getAllByRole("button");
    expect(buttons).toHaveLength(7);
  });

  it("顯示所有狀態按鈕標籤", () => {
    render(<App />);
    expect(screen.getByText("Idle")).toBeInTheDocument();
    expect(screen.getByText("Working")).toBeInTheDocument();
    expect(screen.getByText("Thinking")).toBeInTheDocument();
    expect(screen.getByText("Waiting Input")).toBeInTheDocument();
    expect(screen.getByText("Waiting Confirm")).toBeInTheDocument();
    expect(screen.getByText("Completed")).toBeInTheDocument();
    expect(screen.getByText("Error")).toBeInTheDocument();
  });

  // === Session 顯示 ===

  it("無 session 時顯示空狀態提示", () => {
    render(<App />);
    expect(screen.getByText("尚未偵測到 session")).toBeInTheDocument();
  });

  it("啟動時呼叫 get_sessions 取得初始 session 列表", () => {
    render(<App />);
    expect(mockInvoke).toHaveBeenCalledWith("get_sessions");
  });

  it("有 session 時顯示專案名稱", async () => {
    mockInvoke.mockResolvedValue([
      {
        id: "sess-001",
        agent_type: "claude_code",
        status: "working",
        project_path: "/Users/dev/my-project",
        project_name: "my-project",
        duration_secs: 120,
      },
    ]);

    render(<App />);
    await waitFor(() => {
      expect(screen.getByText("my-project")).toBeInTheDocument();
    });
  });

  // === 事件監聽 ===

  it("啟動時註冊 state-changed 事件監聽器", () => {
    render(<App />);
    expect(mockListen).toHaveBeenCalledWith(
      "state-changed",
      expect.any(Function),
    );
  });
});
