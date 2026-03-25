// [Source] SPEC-001 — AC-8, AC-9
// TDD 測試：SessionList 元件

import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";
import { SessionList } from "../../../src/components/SessionList";
import type { SessionInfo } from "../../../src/store";

function makeSession(overrides: Partial<SessionInfo> = {}): SessionInfo {
  return {
    id: "s1",
    agent_type: "claude_code",
    status: "working",
    project_path: "/Users/dev/my-project",
    project_name: "my-project",
    duration_secs: 180,
    ...overrides,
  };
}

describe("SessionList", () => {
  // === AC-9: Session 列表顯示 ===

  it("AC-9 — 顯示 session 的專案名稱", () => {
    render(
      <SessionList
        sessions={[makeSession()]}
        focusSessionId={null}
        onPin={vi.fn()}
      />,
    );
    expect(screen.getByText("my-project")).toBeInTheDocument();
  });

  it("AC-9 — 顯示 session 的狀態文字", () => {
    render(
      <SessionList
        sessions={[makeSession({ status: "working" })]}
        focusSessionId={null}
        onPin={vi.fn()}
      />,
    );
    expect(screen.getByText(/working/)).toBeInTheDocument();
  });

  it("AC-9 — 顯示 session 的持續時間", () => {
    render(
      <SessionList
        sessions={[makeSession({ duration_secs: 180 })]}
        focusSessionId={null}
        onPin={vi.fn()}
      />,
    );
    expect(screen.getByText(/3m/)).toBeInTheDocument();
  });

  it("AC-9 — 無 session 時顯示空狀態", () => {
    render(
      <SessionList sessions={[]} focusSessionId={null} onPin={vi.fn()} />,
    );
    expect(screen.getByText(/沒有活躍的 session/)).toBeInTheDocument();
  });

  it("AC-9 — 顯示多個 session", () => {
    render(
      <SessionList
        sessions={[
          makeSession({ id: "s1", project_name: "proj-a" }),
          makeSession({ id: "s2", project_name: "proj-b" }),
        ]}
        focusSessionId={null}
        onPin={vi.fn()}
      />,
    );
    expect(screen.getByText("proj-a")).toBeInTheDocument();
    expect(screen.getByText("proj-b")).toBeInTheDocument();
  });

  // === AC-9: 分組排序 ===

  it("AC-9 — 需要注意的 session 排在最上方", () => {
    render(
      <SessionList
        sessions={[
          makeSession({ id: "s1", project_name: "normal", status: "working" }),
          makeSession({
            id: "s2",
            project_name: "urgent",
            status: "waiting_input",
          }),
        ]}
        focusSessionId={null}
        onPin={vi.fn()}
      />,
    );
    const items = screen.getAllByRole("listitem");
    expect(items[0]).toHaveTextContent("urgent");
    expect(items[1]).toHaveTextContent("normal");
  });

  it("AC-9 — error session 排在 working 之前", () => {
    render(
      <SessionList
        sessions={[
          makeSession({ id: "s1", project_name: "active", status: "working" }),
          makeSession({
            id: "s2",
            project_name: "broken",
            status: "error",
          }),
        ]}
        focusSessionId={null}
        onPin={vi.fn()}
      />,
    );
    const items = screen.getAllByRole("listitem");
    expect(items[0]).toHaveTextContent("broken");
    expect(items[1]).toHaveTextContent("active");
  });

  // === AC-8: 釘選 session ===

  it("AC-8 — 點擊 session 觸發 onPin 回呼", async () => {
    const onPin = vi.fn();
    render(
      <SessionList
        sessions={[makeSession({ id: "s1" })]}
        focusSessionId={null}
        onPin={onPin}
      />,
    );

    const item = screen.getByRole("listitem");
    await userEvent.click(item);
    expect(onPin).toHaveBeenCalledWith("s1");
  });

  it("AC-8 — 焦點 session 標記為 active", () => {
    render(
      <SessionList
        sessions={[makeSession({ id: "s1" })]}
        focusSessionId="s1"
        onPin={vi.fn()}
      />,
    );
    const item = screen.getByRole("listitem");
    expect(item).toHaveAttribute("data-active", "true");
  });
});
