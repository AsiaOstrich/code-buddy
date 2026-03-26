#!/usr/bin/env node
// Code Buddy CLI launcher — 偵測平台，啟動對應 binary
const { execFileSync } = require("child_process");
const { platform } = require("../platform");

const pkg = platform();
if (!pkg) {
  console.error(
    `[code-buddy] Unsupported platform: ${process.platform}-${process.arch}`
  );
  process.exit(1);
}

// 允許環境變數覆蓋 binary 路徑
const binPath =
  process.env.CODE_BUDDY_BINARY_PATH ||
  require.resolve(`${pkg.name}/${pkg.bin}`);

try {
  execFileSync(binPath, process.argv.slice(2), { stdio: "inherit" });
} catch (err) {
  process.exit(err.status || 1);
}
