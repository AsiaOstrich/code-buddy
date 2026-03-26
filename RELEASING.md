# 發布流程 / Release Process

## 前置條件

- [ ] `main` branch CI 全綠
- [ ] 所有要進這個版本的 PR 已合併

## 發布步驟

### 1. 更新版本號

三個檔案的版本號必須同步：

```bash
# package.json
"version": "0.3.0"

# src-tauri/Cargo.toml
version = "0.3.0"

# src-tauri/tauri.conf.json（如果有 version 欄位）
```

> **注意**：`Cargo.lock` 會在建置時自動更新，不需要手動改。

### 2. 提交版本號變更

```bash
git add package.json src-tauri/Cargo.toml
git commit -m "chore(release): Bump version to 0.3.0. 版本號更新至 0.3.0."
git push
```

### 3. 等 CI 通過

```bash
gh run list --limit 1
# 確認 conclusion: success
```

### 4. 打 Tag 並推送

```bash
git tag v0.3.0
git push origin v0.3.0
```

這會觸發 `.github/workflows/release.yml`，自動：
- 跑測試（Rust + Frontend）
- 跨平台建置（macOS arm64、macOS x86_64、Linux x86_64、Windows x86_64）
- 上傳安裝檔到 GitHub Releases（Draft 狀態）
- 發布平台套件到 npm（`@asiaostrich/code-buddy-*`）
- 發布主套件到 npm（`@asiaostrich/code-buddy`）

> **npm 發布前置條件**：需要在 GitHub repo Settings → Secrets 設定 `NPM_TOKEN`。
> 取得方式：`npm token create` 或在 npmjs.com 帳號設定中建立。

### 5. 追蹤 Release Workflow

```bash
gh run list --workflow=release.yml --limit 1
gh run watch <run-id>
```

### 6. 檢查 Release Assets

```bash
gh release view v0.3.0 --json assets --jq '.assets[] | "\(.name) \(.size)"'
```

預期產出：

| 檔案 | 平台 |
|------|------|
| `Code.Buddy_*_aarch64.dmg` | macOS Apple Silicon |
| `Code.Buddy_*_amd64.AppImage` | Linux x86_64 |
| `Code.Buddy_*_amd64.deb` | Linux x86_64 (Debian/Ubuntu) |
| `Code.Buddy_*-1.x86_64.rpm` | Linux x86_64 (Fedora/RHEL) |
| `Code.Buddy_*_x64-setup.exe` | Windows x86_64 |
| `Code.Buddy_*_x64_en-US.msi` | Windows x86_64 |

### 7. 撰寫 Release Notes 並發布

到 GitHub Releases 頁面：
1. 編輯 Draft release
2. 撰寫 Release Notes（或用 GitHub 的 auto-generate）
3. 取消勾選 "Set as a pre-release"（正式版時）
4. 點擊 "Publish release"

```bash
# 或用 CLI
gh release edit v0.3.0 --draft=false --notes "Release notes here"
```

### 8. 更新 Homebrew Formula（如適用）

更新 `AsiaOstrich/homebrew-tap` repo 中的 Formula：
- 版本號
- 下載 URL
- SHA256 checksum

```bash
# 取得 checksum
curl -sL <asset-url> | shasum -a 256
```

## 已知問題

### macOS x86_64 DMG 打包失敗

GitHub Actions 的 `macos-latest` 是 ARM (Apple Silicon) runner。
cross-compile 到 `x86_64-apple-darwin` 時，binary 編譯成功但 DMG 打包會失敗。

**現狀**：macOS x86_64 使用者可從 `.app.tar.gz` 安裝。
**未來**：等 GitHub 提供 Intel macOS runner 或改用 universal binary。

### 版本號不同步

Tauri 產出的安裝檔檔名取自 `Cargo.toml` 的 version，不是 git tag。
**必須**在打 tag 前先更新版本號，否則檔名會是舊版本。

## 回滾

如果發布有問題：

```bash
# 刪除 Release
gh release delete v0.3.0 --yes

# 刪除 Tag
git tag -d v0.3.0
git push origin --delete v0.3.0
```

## 清理測試 Release

```bash
# 刪除 alpha/beta 測試 release
gh release delete v0.3.0-alpha --yes
git tag -d v0.3.0-alpha
git push origin --delete v0.3.0-alpha
```
