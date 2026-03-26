// [Derived] SPEC-003 — 浮動圓形動畫視窗 TDD 測試

import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen } from "@testing-library/react";

// Mock lottie-react
vi.mock("lottie-react", () => ({
  default: (props: Record<string, unknown>) => (
    <div
      data-testid="lottie"
      data-loop={String(props.loop ?? true)}
    />
  ),
}));

import FloatApp from "../../../src/FloatApp";

describe("FloatApp", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  // === AC-3: 圓形裁切 ===

  describe("AC-3 — 圓形裁切", () => {
    it("容器為圓形（border-radius: 50%）", () => {
      render(<FloatApp status="idle" />);
      const container = screen.getByTestId("float-container");
      expect(container).toHaveStyle({ borderRadius: "50%" });
    });

    it("overflow 設為 hidden", () => {
      render(<FloatApp status="idle" />);
      const container = screen.getByTestId("float-container");
      expect(container).toHaveStyle({ overflow: "hidden" });
    });

    it("容器大小為 80x80", () => {
      render(<FloatApp status="idle" />);
      const container = screen.getByTestId("float-container");
      expect(container).toHaveStyle({ width: "80px", height: "80px" });
    });
  });

  // === AC-4: 動畫跟隨狀態 ===

  describe("AC-4 — 動畫跟隨狀態切換", () => {
    it("顯示 Lottie 動畫元件", () => {
      render(<FloatApp status="working" />);
      expect(screen.getByTestId("lottie")).toBeInTheDocument();
    });

    it("動畫循環播放", () => {
      render(<FloatApp status="idle" />);
      expect(screen.getByTestId("lottie")).toHaveAttribute("data-loop", "true");
    });

    it("未知狀態降級為 idle", () => {
      render(<FloatApp status="unknown_status" />);
      expect(screen.getByTestId("lottie")).toBeInTheDocument();
    });
  });

  // === AC-8: 狀態色邊框 ===

  describe("AC-8 — 狀態色邊框", () => {
    const STATUS_COLORS: Record<string, string> = {
      idle: "rgb(160, 160, 160)",
      working: "rgb(30, 120, 255)",
      thinking: "rgb(160, 100, 255)",
      waiting_input: "rgb(255, 180, 0)",
      waiting_confirm: "rgb(255, 220, 0)",
      completed: "rgb(50, 200, 80)",
      error: "rgb(240, 50, 50)",
    };

    for (const [status, color] of Object.entries(STATUS_COLORS)) {
      it(`${status} 狀態邊框為 ${color}`, () => {
        render(<FloatApp status={status} />);
        const container = screen.getByTestId("float-container");
        expect(container.style.borderColor).toBe(color);
      });
    }

    it("邊框寬度為 3px", () => {
      render(<FloatApp status="idle" />);
      const container = screen.getByTestId("float-container");
      expect(container.style.borderWidth).toBe("3px");
    });
  });
});
