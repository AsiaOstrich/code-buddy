#!/usr/bin/env node
/**
 * 產生 Code Buddy placeholder Lottie 動畫
 *
 * 每種 AgentStatus 對應一個簡單幾何動畫，未來可由設計師替換為鴕鳥角色動畫。
 * 執行方式：node scripts/generate-placeholder-animations.mjs
 */

import { writeFileSync } from "node:fs";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const OUT_DIR = join(__dirname, "..", "src", "animations");

// === Lottie JSON 建構工具 ===

function base(name, op) {
  return { v: "5.7.4", fr: 30, ip: 0, op, w: 200, h: 200, nm: name, ddd: 0, assets: [], layers: [] };
}

function staticVal(v) {
  return { a: 0, k: v };
}

function animated(keyframes) {
  return { a: 1, k: keyframes };
}

/** 建立 keyframe（Lottie 格式） */
function kf(t, s, easeIn, easeOut) {
  const f = { t, s: Array.isArray(s) ? s : [s] };
  if (easeIn && easeOut) {
    f.i = easeIn;
    f.o = easeOut;
  }
  return f;
}

const easeSmooth = {
  i: { x: [0.42], y: [1] },
  o: { x: [0.58], y: [0] },
};

const easeInOut = {
  i: { x: [0.42], y: [1] },
  o: { x: [0.58], y: [0] },
};

function transform(overrides = {}) {
  return {
    o: staticVal(100),
    r: staticVal(0),
    p: staticVal([100, 100, 0]),
    a: staticVal([0, 0, 0]),
    s: staticVal([100, 100, 100]),
    ...overrides,
  };
}

function shapeLayer(name, shapes, ks, ip = 0, op = 60) {
  return {
    ty: 4, nm: name, ind: 0, ip, op, st: 0,
    ks: transform(ks),
    shapes,
  };
}

function ellipse(size, pos = [0, 0]) {
  return { ty: "el", nm: "ellipse", p: staticVal(pos), s: staticVal([size, size]) };
}

function rect(w, h, pos = [0, 0], r = 0) {
  return { ty: "rc", nm: "rect", p: staticVal(pos), s: staticVal([w, h]), r: staticVal(r) };
}

function fill(color, opacity = 100) {
  return { ty: "fl", nm: "fill", c: staticVal(color), o: staticVal(opacity) };
}

function stroke(color, width = 6, opacity = 100) {
  return { ty: "st", nm: "stroke", c: staticVal(color), o: staticVal(opacity), w: staticVal(width), lc: 2, lj: 2 };
}

function group(items, name = "group") {
  return { ty: "gr", nm: name, it: [...items, { ty: "tr", p: staticVal([0, 0]), a: staticVal([0, 0]), s: staticVal([100, 100]), r: staticVal(0), o: staticVal(100) }] };
}

function trimPath(start, end, offset = 0) {
  return { ty: "tm", nm: "trim", s: start, e: end, o: staticVal(offset), m: 1 };
}

function shapePath(vertices, closed = false) {
  return {
    ty: "sh", nm: "path",
    ks: staticVal({
      c: closed,
      v: vertices,
      i: vertices.map(() => [0, 0]),
      o: vertices.map(() => [0, 0]),
    }),
  };
}

// SPEC-001 配色（RGBA 0-1）
const COLORS = {
  gray:   [0.6, 0.6, 0.6, 1],
  blue:   [0.13, 0.59, 0.95, 1],
  purple: [0.61, 0.15, 0.69, 1],
  yellow: [1, 0.76, 0.03, 1],
  orange: [1, 0.6, 0, 1],
  green:  [0.3, 0.69, 0.31, 1],
  red:    [0.96, 0.26, 0.21, 1],
};

// === 7 個動畫產生函式 ===

/** idle: 圓形緩慢脈動呼吸 (90 幀, 3s loop) */
function generateIdle() {
  const anim = base("idle", 90);
  const layer = shapeLayer("pulse", [
    group([
      ellipse(60),
      fill(COLORS.gray),
    ]),
  ], {
    s: animated([
      kf(0, [100, 100, 100], easeSmooth.i, easeSmooth.o),
      kf(45, [120, 120, 100], easeSmooth.i, easeSmooth.o),
      kf(90, [100, 100, 100]),
    ]),
  }, 0, 90);
  anim.layers.push(layer);
  return anim;
}

/** working: 三個圓點依序彈跳 (30 幀, 1s loop) */
function generateWorking() {
  const anim = base("working", 30);
  const dotSize = 20;
  const spacing = 35;

  for (let i = 0; i < 3; i++) {
    const x = 100 + (i - 1) * spacing;
    const delay = i * 5;
    const layer = shapeLayer(`dot${i}`, [
      group([
        ellipse(dotSize),
        fill(COLORS.blue),
      ]),
    ], {
      p: animated([
        kf(delay % 30, [x, 110, 0], easeInOut.i, easeInOut.o),
        kf((delay + 8) % 30, [x, 80, 0], easeInOut.i, easeInOut.o),
        kf((delay + 15) % 30, [x, 110, 0]),
        kf(30, [x, 110, 0]),
      ]),
    }, 0, 30);
    layer.ind = i;
    anim.layers.push(layer);
  }
  return anim;
}

/** thinking: 三個泡泡逐漸浮上 (60 幀, 2s loop) */
function generateThinking() {
  const anim = base("thinking", 60);
  const sizes = [14, 22, 34];
  const startX = [85, 95, 108];
  const startY = [140, 120, 100];

  for (let i = 0; i < 3; i++) {
    const delay = i * 12;
    const layer = shapeLayer(`bubble${i}`, [
      group([
        ellipse(sizes[i]),
        fill(COLORS.purple, 70),
        stroke(COLORS.purple, 2),
      ]),
    ], {
      p: animated([
        kf(delay, [startX[i], startY[i], 0], easeSmooth.i, easeSmooth.o),
        kf(delay + 25, [startX[i], startY[i] - 40, 0], easeSmooth.i, easeSmooth.o),
        kf(delay + 30, [startX[i], startY[i] - 40, 0]),
      ]),
      o: animated([
        kf(delay, [0], easeSmooth.i, easeSmooth.o),
        kf(delay + 5, [100], easeSmooth.i, easeSmooth.o),
        kf(delay + 25, [100], easeSmooth.i, easeSmooth.o),
        kf(delay + 30, [0]),
        kf(60, [0]),
      ]),
    }, 0, 60);
    layer.ind = i;
    anim.layers.push(layer);
  }
  return anim;
}

/** waiting_input: 閃爍脈動的圓形 + 驚嘆號 (40 幀, 1.3s loop) */
function generateWaitingInput() {
  const anim = base("waiting_input", 40);

  // 背景圓形脈動
  const bg = shapeLayer("bg-pulse", [
    group([
      ellipse(80),
      fill(COLORS.yellow, 30),
    ]),
  ], {
    s: animated([
      kf(0, [100, 100, 100], easeSmooth.i, easeSmooth.o),
      kf(20, [115, 115, 100], easeSmooth.i, easeSmooth.o),
      kf(40, [100, 100, 100]),
    ]),
  }, 0, 40);
  bg.ind = 0;

  // 驚嘆號的直線部分
  const exclamLine = shapeLayer("excl-line", [
    group([
      shapePath([[-3, -25], [3, -25], [2, 8], [-2, 8]], true),
      fill(COLORS.yellow),
    ]),
  ], {
    o: animated([
      kf(0, [100], easeSmooth.i, easeSmooth.o),
      kf(15, [40], easeSmooth.i, easeSmooth.o),
      kf(30, [100]),
      kf(40, [100]),
    ]),
  }, 0, 40);
  exclamLine.ind = 1;

  // 驚嘆號的點
  const exclamDot = shapeLayer("excl-dot", [
    group([
      ellipse(8, [0, 18]),
      fill(COLORS.yellow),
    ]),
  ], {
    o: animated([
      kf(0, [100], easeSmooth.i, easeSmooth.o),
      kf(15, [40], easeSmooth.i, easeSmooth.o),
      kf(30, [100]),
      kf(40, [100]),
    ]),
  }, 0, 40);
  exclamDot.ind = 2;

  anim.layers.push(bg, exclamLine, exclamDot);
  return anim;
}

/** waiting_confirm: 盾牌形狀旋轉脈動 (45 幀, 1.5s loop) */
function generateWaitingConfirm() {
  const anim = base("waiting_confirm", 45);

  // 盾牌形狀（簡化為上方平坦、下方尖角的五邊形）
  const shield = shapeLayer("shield", [
    group([
      shapePath([
        [0, -35], [30, -20], [25, 15], [0, 35], [-25, 15], [-30, -20],
      ], true),
      fill(COLORS.orange, 60),
      stroke(COLORS.orange, 4),
    ]),
  ], {
    s: animated([
      kf(0, [100, 100, 100], easeSmooth.i, easeSmooth.o),
      kf(22, [110, 110, 100], easeSmooth.i, easeSmooth.o),
      kf(45, [100, 100, 100]),
    ]),
    r: animated([
      kf(0, [0], easeSmooth.i, easeSmooth.o),
      kf(12, [-8], easeSmooth.i, easeSmooth.o),
      kf(33, [8], easeSmooth.i, easeSmooth.o),
      kf(45, [0]),
    ]),
  }, 0, 45);

  anim.layers.push(shield);
  return anim;
}

/** completed: 打勾 draw-on 動畫 (30 幀, 1s once) */
function generateCompleted() {
  const anim = base("completed", 30);

  // 背景圓形淡入
  const bg = shapeLayer("bg", [
    group([
      ellipse(80),
      fill(COLORS.green, 20),
    ]),
  ], {
    o: animated([
      kf(0, [0], easeSmooth.i, easeSmooth.o),
      kf(10, [100]),
      kf(30, [100]),
    ]),
    s: animated([
      kf(0, [50, 50, 100], easeSmooth.i, easeSmooth.o),
      kf(10, [100, 100, 100]),
      kf(30, [100, 100, 100]),
    ]),
  }, 0, 30);
  bg.ind = 0;

  // 打勾線條（trim path 動畫）
  const check = shapeLayer("check", [
    group([
      shapePath([[-20, 0], [-5, 18], [22, -18]]),
      stroke(COLORS.green, 8),
      trimPath(staticVal(0), animated([
        kf(8, [0], easeSmooth.i, easeSmooth.o),
        kf(24, [100]),
        kf(30, [100]),
      ])),
    ]),
  ], {}, 0, 30);
  check.ind = 1;

  anim.layers.push(bg, check);
  return anim;
}

/** error: X 標記 + 搖晃 (30 幀, 1s once) */
function generateError() {
  const anim = base("error", 30);

  // 背景圓形
  const bg = shapeLayer("bg", [
    group([
      ellipse(80),
      fill(COLORS.red, 20),
    ]),
  ], {
    o: animated([
      kf(0, [0], easeSmooth.i, easeSmooth.o),
      kf(8, [100]),
      kf(30, [100]),
    ]),
    s: animated([
      kf(0, [50, 50, 100], easeSmooth.i, easeSmooth.o),
      kf(8, [100, 100, 100]),
      kf(30, [100, 100, 100]),
    ]),
  }, 0, 30);
  bg.ind = 0;

  // X 線條 1
  const x1 = shapeLayer("x1", [
    group([
      shapePath([[-16, -16], [16, 16]]),
      stroke(COLORS.red, 8),
      trimPath(staticVal(0), animated([
        kf(6, [0], easeSmooth.i, easeSmooth.o),
        kf(16, [100]),
        kf(30, [100]),
      ])),
    ]),
  ], {}, 0, 30);
  x1.ind = 1;

  // X 線條 2
  const x2 = shapeLayer("x2", [
    group([
      shapePath([[16, -16], [-16, 16]]),
      stroke(COLORS.red, 8),
      trimPath(staticVal(0), animated([
        kf(10, [0], easeSmooth.i, easeSmooth.o),
        kf(20, [100]),
        kf(30, [100]),
      ])),
    ]),
  ], {
    // 搖晃效果
    p: animated([
      kf(18, [100, 100, 0]),
      kf(20, [106, 100, 0]),
      kf(22, [94, 100, 0]),
      kf(24, [104, 100, 0]),
      kf(26, [97, 100, 0]),
      kf(28, [100, 100, 0]),
      kf(30, [100, 100, 0]),
    ]),
  }, 0, 30);
  x2.ind = 2;

  anim.layers.push(bg, x1, x2);
  return anim;
}

// === 產生並寫入檔案 ===

const animations = {
  idle: generateIdle(),
  working: generateWorking(),
  thinking: generateThinking(),
  waiting_input: generateWaitingInput(),
  waiting_confirm: generateWaitingConfirm(),
  completed: generateCompleted(),
  error: generateError(),
};

for (const [name, data] of Object.entries(animations)) {
  const path = join(OUT_DIR, `${name}.json`);
  const json = JSON.stringify(data);
  writeFileSync(path, json);
  const kb = (Buffer.byteLength(json) / 1024).toFixed(1);
  console.log(`  ✓ ${name}.json (${kb} KB)`);
}

console.log("\n完成！7 個 placeholder 動畫已產生。");
