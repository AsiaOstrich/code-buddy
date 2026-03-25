// [Source] SPEC-001 — AC-8, AC-9
// TDD 測試：Zustand Store — 前端狀態管理

import { describe, it, expect, vi, beforeEach } from "vitest";
import { useAppStore } from "../../../src/store";

// Mock Tauri API
const mockInvoke = vi.fn();
const mockListen = vi.fn();

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => mockInvoke(...args),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: (...args: unknown[]) => mockListen(...args),
}));

describe("AppStore", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    useAppStore.setState(useAppStore.getInitialState());
    mockInvoke.mockResolvedValue([]);
    mockListen.mockResolvedValue(vi.fn());
  });

  // === AC-8: 初始狀態 ===

  it("AC-8 — 初始 sessions 為空陣列", () => {
    const { sessions } = useAppStore.getState();
    expect(sessions).toEqual([]);
  });

  it("AC-8 — 初始 effectiveStatus 為 idle", () => {
    const { effectiveStatus } = useAppStore.getState();
    expect(effectiveStatus).toBe("idle");
  });

  it("AC-8 — 初始 focusSessionId 為 null", () => {
    const { focusSessionId } = useAppStore.getState();
    expect(focusSessionId).toBeNull();
  });

  // === AC-9: init 初始化 ===

  it("AC-9 — init 註冊 state-changed 事件監聽器", async () => {
    await useAppStore.getState().init();
    expect(mockListen).toHaveBeenCalledWith(
      "state-changed",
      expect.any(Function),
    );
  });

  it("AC-9 — init 呼叫 get_sessions 取得快照", async () => {
    await useAppStore.getState().init();
    expect(mockInvoke).toHaveBeenCalledWith("get_sessions");
  });

  it("AC-9 — init 先建立 listener 再拉取快照", async () => {
    const callOrder: string[] = [];
    mockListen.mockImplementation(() => {
      callOrder.push("listen");
      return Promise.resolve(vi.fn());
    });
    mockInvoke.mockImplementation(() => {
      callOrder.push("invoke");
      return Promise.resolve([]);
    });

    await useAppStore.getState().init();
    expect(callOrder).toEqual(["listen", "invoke"]);
  });

  it("AC-9 — init 將快照寫入 sessions", async () => {
    const mockSessions = [
      {
        id: "s1",
        agent_type: "claude_code",
        status: "working",
        project_path: "/path/proj",
        project_name: "proj",
        duration_secs: 60,
      },
    ];
    mockInvoke.mockResolvedValue(mockSessions);

    await useAppStore.getState().init();
    expect(useAppStore.getState().sessions).toEqual(mockSessions);
  });

  // === AC-9: state-changed 事件更新 ===

  it("AC-9 — 收到 state-changed 事件時更新 sessions", async () => {
    let handler: (event: unknown) => void = () => {};
    mockListen.mockImplementation((_event: string, cb: typeof handler) => {
      handler = cb;
      return Promise.resolve(vi.fn());
    });

    await useAppStore.getState().init();

    const newSessions = [
      {
        id: "s1",
        agent_type: "claude_code",
        status: "completed",
        project_path: "/path/proj",
        project_name: "proj",
        duration_secs: 120,
      },
    ];

    handler({
      payload: {
        session_id: "s1",
        status: "completed",
        effective_status: "completed",
        sessions: newSessions,
      },
    });

    expect(useAppStore.getState().sessions).toEqual(newSessions);
    expect(useAppStore.getState().effectiveStatus).toBe("completed");
  });

  it("AC-9 — 收到 state-changed 事件時更新 focusSessionId", async () => {
    let handler: (event: unknown) => void = () => {};
    mockListen.mockImplementation((_event: string, cb: typeof handler) => {
      handler = cb;
      return Promise.resolve(vi.fn());
    });

    await useAppStore.getState().init();

    handler({
      payload: {
        session_id: "s1",
        status: "working",
        effective_status: "working",
        sessions: [
          {
            id: "s1",
            agent_type: "claude_code",
            status: "working",
            project_path: "/p",
            project_name: "p",
            duration_secs: 0,
          },
        ],
      },
    });

    expect(useAppStore.getState().focusSessionId).toBe("s1");
  });

  // === AC-8: pinSession ===

  it("AC-8 — pinSession 呼叫 Tauri invoke", async () => {
    mockInvoke.mockResolvedValue(null);
    await useAppStore.getState().pinSession("s1");
    expect(mockInvoke).toHaveBeenCalledWith("pin_session", {
      sessionId: "s1",
    });
  });

  it("AC-8 — pinSession 更新 focusSessionId", async () => {
    mockInvoke.mockResolvedValue(null);
    await useAppStore.getState().pinSession("s1");
    expect(useAppStore.getState().focusSessionId).toBe("s1");
  });

  // === AC-9: attentionCount ===

  it("AC-9 — attentionCount 計算需要注意的 session 數", () => {
    useAppStore.setState({
      sessions: [
        {
          id: "s1",
          agent_type: "claude_code",
          status: "waiting_input",
          project_path: "/a",
          project_name: "a",
          duration_secs: 0,
        },
        {
          id: "s2",
          agent_type: "claude_code",
          status: "working",
          project_path: "/b",
          project_name: "b",
          duration_secs: 0,
        },
        {
          id: "s3",
          agent_type: "claude_code",
          status: "error",
          project_path: "/c",
          project_name: "c",
          duration_secs: 0,
        },
      ],
    });
    expect(useAppStore.getState().attentionCount()).toBe(2);
  });

  it("AC-9 — attentionCount 無需要注意時回傳 0", () => {
    useAppStore.setState({
      sessions: [
        {
          id: "s1",
          agent_type: "claude_code",
          status: "working",
          project_path: "/a",
          project_name: "a",
          duration_secs: 0,
        },
      ],
    });
    expect(useAppStore.getState().attentionCount()).toBe(0);
  });
});
