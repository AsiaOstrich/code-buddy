// [Source] SPEC-001 — AC-9
// [Generated] TDD 測試骨架：BuddyAnimation 元件
// 測試目標：src/components/BuddyAnimation.tsx

// [TODO] 需安裝 vitest + @testing-library/react
// [TODO] 需 mock lottie-react

import { describe, it, expect } from 'vitest';
// [TODO] import { render } from '@testing-library/react';
// [TODO] import { BuddyAnimation } from '../../../src/components/BuddyAnimation';

describe('BuddyAnimation', () => {

  // === AC-9: 動畫渲染 ===

  it('AC-9 — 根據狀態載入對應 Lottie JSON', () => {
    // [Derived] AC-9: 每種狀態對應不同動畫
    // [TODO] render(<BuddyAnimation status="working" />);
    // [TODO] 驗證載入的是 animations/working.json
  });

  it('AC-9 — working/thinking/idle 動畫為循環播放', () => {
    // [Derived] AC-9: 循環動畫 loop=true
    // [TODO] render(<BuddyAnimation status="working" />);
    // [TODO] 驗證 Lottie loop prop 為 true
  });

  it('AC-9 — completed/error 動畫播放一次後靜止', () => {
    // [Derived] AC-9: 一次性動畫 loop=false
    // [TODO] render(<BuddyAnimation status="completed" />);
    // [TODO] 驗證 Lottie loop prop 為 false
  });

  it('AC-9 — 狀態變化時切換動畫', () => {
    // [Derived] AC-9: 狀態從 working → completed 時動畫切換
    // [TODO] const { rerender } = render(<BuddyAnimation status="working" />);
    // [TODO] rerender(<BuddyAnimation status="completed" />);
    // [TODO] 驗證動畫已切換
  });
});
