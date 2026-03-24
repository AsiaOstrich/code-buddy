// [Source] SPEC-001 — AC-8, AC-9
// [Generated] TDD 測試骨架：SessionList 元件
// 測試目標：src/components/SessionList.tsx

// [TODO] 需安裝 vitest + @testing-library/react

import { describe, it, expect } from 'vitest';
// [TODO] import { render, screen } from '@testing-library/react';
// [TODO] import { SessionList } from '../../../src/components/SessionList';

describe('SessionList', () => {

  // === AC-9: Session 列表顯示 ===

  it('AC-9 — 顯示 session 的專案名稱', () => {
    // [Derived] AC-9: 面板顯示專案名稱
    // [TODO] render(<SessionList sessions={mockSessions} />);
    // [TODO] expect(screen.getByText('my-project')).toBeInTheDocument();
  });

  it('AC-9 — 顯示 session 的狀態文字', () => {
    // [Derived] AC-9: 面板顯示狀態文字
    // [TODO] render(<SessionList sessions={mockSessions} />);
    // [TODO] expect(screen.getByText('工作中')).toBeInTheDocument();
  });

  it('AC-9 — 顯示 session 的持續時間', () => {
    // [Derived] AC-9: 面板顯示持續時間
    // [TODO] render(<SessionList sessions={mockSessions} />);
    // [TODO] expect(screen.getByText(/3m/)).toBeInTheDocument();
  });

  it('AC-9 — 顯示 agent 類型圖示', () => {
    // [Derived] AC-9: Claude Code 顯示 🤖，OpenCode 顯示 🔷
    // [TODO] 驗證 agent_type 對應正確圖示
  });

  // === AC-9: 分組排序 ===

  it('AC-9 — 需要注意的 session 排在最上方', () => {
    // [Derived] AC-9: waiting_input/error session 置頂
    // [TODO] 驗證 DOM 順序
  });

  it('AC-9 — 工作中的 session 排在第二組', () => {
    // [Derived] AC-9: working/thinking 排在中間
    // [TODO] 驗證 DOM 順序
  });
});
