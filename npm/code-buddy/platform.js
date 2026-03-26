// 平台偵測 — 對應平台套件名稱和 binary 路徑
const PLATFORMS = {
  "darwin arm64": {
    name: "@asiaostrich/code-buddy-darwin-arm64",
    bin: "bin/code-buddy",
  },
  "darwin x64": {
    name: "@asiaostrich/code-buddy-darwin-x64",
    bin: "bin/code-buddy",
  },
  "linux x64": {
    name: "@asiaostrich/code-buddy-linux-x64",
    bin: "bin/code-buddy",
  },
  "win32 x64": {
    name: "@asiaostrich/code-buddy-win32-x64",
    bin: "bin/code-buddy.exe",
  },
};

function platform() {
  const key = `${process.platform} ${process.arch}`;
  return PLATFORMS[key] || null;
}

module.exports = { platform, PLATFORMS };
