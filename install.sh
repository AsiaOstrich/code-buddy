#!/bin/sh
# Code Buddy 安裝腳本
# 用法: curl -fsSL https://raw.githubusercontent.com/AsiaOstrich/code-buddy/main/install.sh | sh
set -e

REPO="AsiaOstrich/code-buddy"
INSTALL_DIR="${HOME}/.local/bin"

# 顏色輸出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
CYAN='\033[0;36m'
NC='\033[0m'

info()  { printf "${CYAN}[info]${NC}  %s\n" "$1"; }
ok()    { printf "${GREEN}[ok]${NC}    %s\n" "$1"; }
warn()  { printf "${YELLOW}[warn]${NC}  %s\n" "$1"; }
err()   { printf "${RED}[error]${NC} %s\n" "$1" >&2; exit 1; }

# 偵測 OS
detect_os() {
  case "$(uname -s)" in
    Darwin)  echo "darwin" ;;
    Linux)   echo "linux" ;;
    *)       err "不支援的作業系統: $(uname -s)。Windows 請使用 install.ps1" ;;
  esac
}

# 偵測 CPU 架構
detect_arch() {
  case "$(uname -m)" in
    x86_64|amd64)  echo "x86_64" ;;
    arm64|aarch64) echo "aarch64" ;;
    *)             err "不支援的 CPU 架構: $(uname -m)" ;;
  esac
}

# 取得最新版本號
get_latest_version() {
  if command -v curl >/dev/null 2>&1; then
    curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name"' | sed -E 's/.*"([^"]+)".*/\1/'
  elif command -v wget >/dev/null 2>&1; then
    wget -qO- "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name"' | sed -E 's/.*"([^"]+)".*/\1/'
  else
    err "需要 curl 或 wget"
  fi
}

# 下載檔案
download() {
  local url="$1" dest="$2"
  if command -v curl >/dev/null 2>&1; then
    curl -fsSL "$url" -o "$dest"
  elif command -v wget >/dev/null 2>&1; then
    wget -q "$url" -O "$dest"
  fi
}

main() {
  info "Code Buddy 安裝程式"
  echo ""

  # 偵測平台
  OS=$(detect_os)
  ARCH=$(detect_arch)
  info "偵測到平台: ${OS}-${ARCH}"

  # 取得最新版本
  info "查詢最新版本..."
  VERSION=$(get_latest_version)
  if [ -z "$VERSION" ]; then
    err "無法取得最新版本。請確認網路連線或至 https://github.com/${REPO}/releases 手動下載"
  fi
  info "最新版本: ${VERSION}"

  # 組合下載 URL
  # Tauri 產出的檔名格式依平台而異
  case "${OS}" in
    darwin)
      FILENAME="Code Buddy_${VERSION#v}_${ARCH}.dmg"
      ;;
    linux)
      FILENAME="code-buddy_${VERSION#v}_${ARCH}.AppImage"
      ;;
  esac

  DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${VERSION}/${FILENAME}"

  # 建立安裝目錄
  mkdir -p "$INSTALL_DIR"

  # 下載
  info "下載中: ${FILENAME}..."
  TMPDIR=$(mktemp -d)
  trap 'rm -rf "$TMPDIR"' EXIT

  if ! download "$DOWNLOAD_URL" "${TMPDIR}/${FILENAME}"; then
    err "下載失敗。請確認版本 ${VERSION} 有 ${OS}-${ARCH} 的建置檔"
  fi
  ok "下載完成"

  # 安裝
  case "${OS}" in
    darwin)
      info "macOS: 掛載 DMG 並複製應用程式..."
      MOUNT_POINT=$(hdiutil attach "${TMPDIR}/${FILENAME}" -nobrowse -quiet | grep -o '/Volumes/.*')
      if [ -d "${MOUNT_POINT}" ]; then
        APP_NAME=$(find "${MOUNT_POINT}" -maxdepth 1 -name "*.app" | head -1)
        if [ -n "$APP_NAME" ]; then
          cp -R "$APP_NAME" "/Applications/"
          ok "已安裝至 /Applications/$(basename "$APP_NAME")"
        fi
        hdiutil detach "${MOUNT_POINT}" -quiet
      else
        err "無法掛載 DMG"
      fi
      ;;
    linux)
      chmod +x "${TMPDIR}/${FILENAME}"
      mv "${TMPDIR}/${FILENAME}" "${INSTALL_DIR}/code-buddy"
      ok "已安裝至 ${INSTALL_DIR}/code-buddy"

      # 檢查 PATH
      case ":$PATH:" in
        *":${INSTALL_DIR}:"*) ;;
        *)
          warn "${INSTALL_DIR} 不在 PATH 中"
          echo ""
          echo "  請將以下行加入你的 shell 設定檔 (~/.bashrc 或 ~/.zshrc):"
          echo ""
          echo "    export PATH=\"${INSTALL_DIR}:\$PATH\""
          echo ""
          ;;
      esac
      ;;
  esac

  echo ""
  ok "Code Buddy ${VERSION} 安裝完成！"
  echo ""
  info "下一步：執行 'code-buddy setup' 設定 Claude Code Hook"
}

main "$@"
