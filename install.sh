#!/usr/bin/env bash
# ==============================================================================
# mdview - Official One-Line Installer
# Usage: curl -fsSL https://raw.githubusercontent.com/imyash0722/mdview/main/install.sh | bash
# ==============================================================================

set -euo pipefail

REPO="imyash0722/mdview"
BINARY="mdview"

BOLD="\033[1m"
GREEN="\033[1;32m"
BLUE="\033[1;34m"
YELLOW="\033[1;33m"
RED="\033[1;31m"
RESET="\033[0m"

log_info() { echo -e "${BLUE}==>${RESET} ${BOLD}$1${RESET}"; }
log_success() { echo -e "${GREEN}==>${RESET} ${BOLD}$1${RESET}"; }
log_warn() { echo -e "${YELLOW}Warning:${RESET} $1"; }
log_error() { echo -e "${RED}Error:${RESET} $1" >&2; exit 1; }

echo -e "${BLUE}"
echo "    ███    ███ ██████  ██    ██ ██ ███████ ██     ██ "
echo "    ████  ████ ██   ██ ██    ██ ██ ██      ██     ██ "
echo "    ██ ████ ██ ██   ██ ██    ██ ██ █████   ██  █  ██ "
echo "    ██  ██  ██ ██   ██  ██  ██  ██ ██      ██ ███ ██ "
echo "    ██      ██ ██████    ████   ██ ███████  ███ ███  "
echo -e "${RESET}"

# 1. Detect OS & Architecture
OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"

case "$OS" in
    linux)
        TARGET_OS="unknown-linux-gnu"
        ;;
    darwin)
        TARGET_OS="apple-darwin"
        ;;
    *)
        log_error "Unsupported Operating System: '$OS'. Pre-built releases support Linux and macOS."
        ;;
esac

case "$ARCH" in
    x86_64|amd64)
        TARGET_ARCH="x86_64"
        ;;
    aarch64|arm64)
        TARGET_ARCH="aarch64"
        ;;
    *)
        log_error "Unsupported Architecture: '$ARCH'. Supported architectures: x86_64, aarch64."
        ;;
esac

TARGET="${TARGET_ARCH}-${TARGET_OS}"
log_info "Detected Platform: ${BOLD}${TARGET}${RESET}"

# 2. Query latest release tag
log_info "Fetching latest release information..."
LATEST_TAG=$(curl -s "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name":' | head -n1 | cut -d '"' -f4 || true)

if [ -z "$LATEST_TAG" ]; then
    LATEST_TAG="v1.1.0"
    log_warn "Could not query GitHub API for latest tag, falling back to ${LATEST_TAG}"
fi

ARCHIVE="mdview-${LATEST_TAG}-${TARGET}.tar.gz"
DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${LATEST_TAG}/${ARCHIVE}"

# 3. Download Archive
TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT

log_info "Downloading ${ARCHIVE}..."
if ! curl -fSL --progress-bar "$DOWNLOAD_URL" -o "${TMP_DIR}/${ARCHIVE}"; then
    log_error "Failed to download release archive from ${DOWNLOAD_URL}"
fi

# 4. Extract
log_info "Extracting..."
tar -xzf "${TMP_DIR}/${ARCHIVE}" -C "$TMP_DIR"
EXTRACTED_DIR="${TMP_DIR}/mdview-${LATEST_TAG}-${TARGET}"

# 5. Determine install directory
INSTALL_DIR=""
if [ -w "/usr/local/bin" ]; then
    INSTALL_DIR="/usr/local/bin"
elif [ -d "$HOME/.local/bin" ] || mkdir -p "$HOME/.local/bin" 2>/dev/null; then
    INSTALL_DIR="$HOME/.local/bin"
else
    INSTALL_DIR="/usr/local/bin"
fi

log_info "Installing '${BINARY}' to ${INSTALL_DIR}..."
if [ -w "$INSTALL_DIR" ]; then
    install -m 755 "${EXTRACTED_DIR}/${BINARY}" "${INSTALL_DIR}/${BINARY}"
else
    sudo install -m 755 "${EXTRACTED_DIR}/${BINARY}" "${INSTALL_DIR}/${BINARY}"
fi

# 6. Install shell completions if directories exist
# Zsh
if [ -d "$HOME/.config/zsh/completions" ]; then
    cp -f "${EXTRACTED_DIR}/completions/zsh/_mdview" "$HOME/.config/zsh/completions/_mdview" 2>/dev/null || true
    cp -f "${EXTRACTED_DIR}/completions/zsh/_md" "$HOME/.config/zsh/completions/_md" 2>/dev/null || true
fi

# Fish
if [ -d "$HOME/.config/fish/completions" ]; then
    cp -f "${EXTRACTED_DIR}/completions/fish/mdview.fish" "$HOME/.config/fish/completions/mdview.fish" 2>/dev/null || true
    cp -f "${EXTRACTED_DIR}/completions/fish/md.fish" "$HOME/.config/fish/completions/md.fish" 2>/dev/null || true
fi

# Bash
if [ -d "$HOME/.local/share/bash-completion/completions" ]; then
    cp -f "${EXTRACTED_DIR}/completions/bash/mdview.bash" "$HOME/.local/share/bash-completion/completions/mdview" 2>/dev/null || true
fi

log_success "Successfully installed mdview ${LATEST_TAG} to ${INSTALL_DIR}/${BINARY}"

# Path warning if needed
if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
    log_warn "'${INSTALL_DIR}' is not currently in your \$PATH."
    echo "Add it to your shell rc file (e.g. export PATH=\"\$HOME/.local/bin:\$PATH\")"
fi

echo ""
echo -e "${BOLD}Get started by viewing any Markdown file:${RESET}"
echo -e "  ${GREEN}mdview README.md${RESET}"
echo -e "  ${GREEN}alias md=\"mdview\"${RESET}"
echo ""
