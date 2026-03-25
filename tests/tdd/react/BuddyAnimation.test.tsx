// [Source] SPEC-001 — AC-9
// TDD 測試：BuddyAnimation 元件 — Lottie 動畫角色

import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import { BuddyAnimation } from "../../../src/components/BuddyAnimation";

// Mock lottie-react
const MockLottie = vi.fn((props: Record<string, unknown>) => (
  <div data-testid="lottie" data-loop={String(props.loop)} />
));

vi.mock("lottie-react", () => ({
  default: (props: Record<string, unknown>) => MockLottie(props),
}));

describe("BuddyAnimation", () => {
  // === AC-9: 動畫渲染 ===

  it("AC-9 — 渲染 Lottie 元件", () => {
    render(<BuddyAnimation status="idle" />);
    expect(screen.getByTestId("lottie")).toBeInTheDocument();
  });

  it("AC-9 — 根據狀態傳遞對應動畫資料", () => {
    render(<BuddyAnimation status="working" />);
    expect(MockLottie).toHaveBeenCalledWith(
      expect.objectContaining({
        animationData: expect.any(Object),
      }),
    );
  });

  it("AC-9 — 不同狀態傳遞不同動畫資料", () => {
    const { unmount } = render(<BuddyAnimation status="working" />);
    const workingData = MockLottie.mock.calls.at(-1)?.[0]?.animationData;
    unmount();

    MockLottie.mockClear();
    render(<BuddyAnimation status="error" />);
    const errorData = MockLottie.mock.calls.at(-1)?.[0]?.animationData;

    expect(workingData).not.toBe(errorData);
  });

  // === AC-9: 循環/一次性播放 ===

  it("AC-9 — working 動畫為循環播放", () => {
    render(<BuddyAnimation status="working" />);
    expect(screen.getByTestId("lottie")).toHaveAttribute("data-loop", "true");
  });

  it("AC-9 — thinking 動畫為循環播放", () => {
    render(<BuddyAnimation status="thinking" />);
    expect(screen.getByTestId("lottie")).toHaveAttribute("data-loop", "true");
  });

  it("AC-9 — idle 動畫為循環播放", () => {
    render(<BuddyAnimation status="idle" />);
    expect(screen.getByTestId("lottie")).toHaveAttribute("data-loop", "true");
  });

  it("AC-9 — completed 動畫循環播放", () => {
    render(<BuddyAnimation status="completed" />);
    expect(screen.getByTestId("lottie")).toHaveAttribute("data-loop", "true");
  });

  it("AC-9 — error 動畫循環播放", () => {
    render(<BuddyAnimation status="error" />);
    expect(screen.getByTestId("lottie")).toHaveAttribute("data-loop", "true");
  });

  // === AC-9: 狀態切換 ===

  it("AC-9 — 狀態變化時切換動畫", () => {
    const { rerender } = render(<BuddyAnimation status="working" />);
    const firstData = MockLottie.mock.calls.at(-1)?.[0]?.animationData;

    rerender(<BuddyAnimation status="completed" />);
    const secondData = MockLottie.mock.calls.at(-1)?.[0]?.animationData;

    expect(firstData).not.toBe(secondData);
  });
});
