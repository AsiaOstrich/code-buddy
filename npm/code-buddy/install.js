// Code Buddy postinstall — 驗證平台套件已安裝
const { platform, arch } = require("./platform");

function validate() {
  const pkg = platform();
  if (!pkg) {
    console.warn(
      `[code-buddy] Warning: unsupported platform ${process.platform}-${process.arch}. ` +
        `Code Buddy may not work correctly.`
    );
    return;
  }

  try {
    require.resolve(`${pkg.name}/${pkg.bin}`);
  } catch {
    console.error(
      `[code-buddy] Error: platform package ${pkg.name} is not installed.\n` +
        `Try reinstalling: npm install -g @asiaostrich/code-buddy`
    );
    process.exit(1);
  }

  console.log(`[code-buddy] Installed for ${process.platform}-${process.arch}`);
}

validate();
