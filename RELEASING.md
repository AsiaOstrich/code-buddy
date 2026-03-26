# 發布流程 / Release Process

## 首次設定（只需做一次）

### npm 發布

1. 在 [npmjs.com](https://www.npmjs.com/) 建立 `@asiaostrich` organization
2. 在 npmjs.com → Access Tokens 建立 Automation token
3. 到 GitHub repo → Settings → Secrets and variables → Actions → 新增 secret：
   - Name: `NPM_TOKEN`
   - Value: 上一步取得的 token

驗證：
```bash
npm whoami  # 確認已登入
npm org ls asiaostrich  # 確認 org 存在
```

### Homebrew tap

1. 建立 GitHub repo: `AsiaOstrich/homebrew-tap`
2. 在 repo 中建立 `Formula/code-buddy.rb`（參考 `homebrew/code-buddy.rb.template`）

驗證：
```bash
brew tap asiaostrich/tap
brew search code-buddy
```

### GitHub Actions 權限

確認 repo → Settings → Actions → General：
- Workflow permissions: **Read and write permissions**（Release 需要寫入權限）

---

## 發布 Checklist

每次發布都走這個清單，照順序打勾：

### 準備

- [ ] `main` branch CI 全綠
- [ ] 所有要進這個版本的 PR 已合併
- [ ] 決定版本號（遵循 [SemVer](https://semver.org/)）

### 步驟 1：更新版本號

**兩個檔案的版本號必須同步**（這是最容易出錯的地方）：

```bash
# 1. package.json — 修改 "version" 欄位
# 2. src-tauri/Cargo.toml — 修改 version 欄位
```

> ⚠️ **版本號不同步會導致**：安裝檔檔名是舊版本（Tauri 取 Cargo.toml 的 version）

> 💡 `npm/` 下的 package.json 版本號**不需要手動改**，CI 會自動從 git tag 設定。

### 步驟 2：提交並等 CI

```bash
git add package.json src-tauri/Cargo.toml
git commit -m "chore(release): Bump version to 0.3.0. 版本號更新至 0.3.0."
git push

# 等 CI 全綠
gh run list --limit 1
# 確認 conclusion: success
```

### 步驟 3：打 Tag

```bash
git tag v0.3.0
git push origin v0.3.0
```

這會自動觸發 release workflow，執行：
1. ✅ 測試（Rust + Frontend）
2. ✅ 跨平台建置（macOS arm64/x86_64, Linux x86_64, Windows x86_64）
3. ✅ 上傳安裝檔到 GitHub Releases（Draft）
4. ✅ 發布 4 個平台套件到 npm（`@asiaostrich/code-buddy-*`）
5. ✅ 發布主套件到 npm（`@asiaostrich/code-buddy`）

### 步驟 4：追蹤進度

```bash
gh run list --workflow=release.yml --limit 1
gh run watch <run-id>
```

### 步驟 5：驗證產出

#### GitHub Releases

```bash
gh release view v0.3.0 --json assets --jq '.assets[] | "\(.name) \(.size)"'
```

預期檔案：

| 檔案 | 平台 |
|------|------|
| `Code.Buddy_*_aarch64.dmg` | macOS Apple Silicon |
| `Code.Buddy_*_amd64.AppImage` | Linux x86_64 |
| `Code.Buddy_*_amd64.deb` | Linux x86_64 (Debian/Ubuntu) |
| `Code.Buddy_*-1.x86_64.rpm` | Linux x86_64 (Fedora/RHEL) |
| `Code.Buddy_*_x64-setup.exe` | Windows x86_64 |
| `Code.Buddy_*_x64_en-US.msi` | Windows x86_64 |

#### npm

```bash
npm view @asiaostrich/code-buddy version
npm view @asiaostrich/code-buddy-darwin-arm64 version
# 都應該顯示 0.3.0
```

#### 安裝測試

```bash
# npm
npm install -g @asiaostrich/code-buddy
code-buddy --version

# curl
curl -fsSL https://raw.githubusercontent.com/AsiaOstrich/code-buddy/main/install.sh | sh
```

### 步驟 6：發布 GitHub Release

```bash
gh release edit v0.3.0 --draft=false --notes "Release notes here"
# 或到 GitHub 網頁編輯 Release Notes 後按 Publish
```

### 步驟 7：更新 Homebrew Formula

```bash
# 取得 checksum
curl -sL https://github.com/AsiaOstrich/code-buddy/releases/download/v0.3.0/Code.Buddy_0.3.0_aarch64.dmg | shasum -a 256

# 更新 AsiaOstrich/homebrew-tap repo 的 Formula/code-buddy.rb
# - version
# - url
# - sha256
```

---

## 排錯

### Release workflow 失敗

```bash
# 查看失敗日誌
gh run view <run-id> --log-failed
```

常見原因：
| 症狀 | 原因 | 修復 |
|------|------|------|
| `npm publish` 403 | `NPM_TOKEN` 未設定或過期 | 重新建立 token 並更新 secret |
| `npm publish` 404 | `@asiaostrich` org 不存在 | 到 npmjs.com 建立 org |
| macOS x86_64 DMG 失敗 | ARM runner 無法打包 x86 DMG | 已知問題，binary 仍會成功建置 |
| 版本號不符 | 忘記步驟 1 更新版本號 | 刪除 tag，修版本號，重新 tag |

### 版本號搞錯了

```bash
# 刪除 Release + Tag，重來
gh release delete v0.3.0 --yes
git tag -d v0.3.0
git push origin --delete v0.3.0

# npm 已發布的版本無法刪除，但可以 deprecate
npm deprecate @asiaostrich/code-buddy@0.3.0 "版本號錯誤，請使用 0.3.1"
```

### 回滾

```bash
# 刪除 GitHub Release
gh release delete v0.3.0 --yes

# 刪除 Tag
git tag -d v0.3.0
git push origin --delete v0.3.0

# npm unpublish（只在發布後 72 小時內有效）
npm unpublish @asiaostrich/code-buddy@0.3.0
```

---

## 清理測試 Release

```bash
gh release delete v0.3.0-alpha --yes
git tag -d v0.3.0-alpha
git push origin --delete v0.3.0-alpha
```
