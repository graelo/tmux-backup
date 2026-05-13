#!/usr/bin/env bash

# tmux-backup binary installer
# Downloads the appropriate pre-built binary from GitHub releases.
# Automatically re-downloads when TPM updates the plugin (version mismatch).

set -euo pipefail

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
REPO="graelo/tmux-backup"
BINARY="tmux-backup"
BINARY_PATH="${SCRIPT_DIR}/${BINARY}"
VERSION_FILE="${SCRIPT_DIR}/.installed-version"
RELEASES_URL="https://github.com/${REPO}/releases"

FORCE=false
QUIET=false

log() { $QUIET || echo "[$(date '+%H:%M:%S')] tmux-backup: $1" >&2; }
die() { echo "[$(date '+%H:%M:%S')] tmux-backup: $1" >&2; exit 1; }

# --- Version check ---

plugin_version() {
    git -C "$SCRIPT_DIR" describe --tags --abbrev=0 2>/dev/null || echo "unknown"
}

installed_version() {
    cat "$VERSION_FILE" 2>/dev/null || echo "unknown"
}

needs_update() {
    $FORCE && return 0
    [[ ! -x "$BINARY_PATH" ]] && return 0
    [[ "$(plugin_version)" != "$(installed_version)" ]]
}

# --- Download helpers ---

fetch() {
    if command -v curl >/dev/null 2>&1; then
        curl -fsSL "$@"
    elif command -v wget >/dev/null 2>&1; then
        wget -qO- "$@"
    else
        die "Error: curl or wget required"
    fi
}

download() {
    if command -v curl >/dev/null 2>&1; then
        curl -fSL -o "$2" "$1" 2>/dev/null
    else
        wget -qO "$2" "$1" 2>/dev/null
    fi
}

# --- Core logic ---

install() {
    needs_update || { log "Up to date ($(installed_version))"; return 0; }

    # Detect platform
    local os arch
    case "$(uname -s)" in
        Darwin) os="darwin" ;; Linux) os="linux" ;; *) die "Unsupported OS: $(uname -s). Install manually: ${RELEASES_URL}" ;;
    esac
    case "$(uname -m)" in
        x86_64|amd64) arch="x86_64" ;; arm64|aarch64) arch="aarch64" ;; *) die "Unsupported arch: $(uname -m). Install manually: ${RELEASES_URL}" ;;
    esac

    # Build expected standalone-binary asset name
    local asset
    case "$os" in
        darwin) asset="${BINARY}-${arch}-apple-darwin" ;;
        linux)  asset="${BINARY}-${arch}-unknown-linux-musl" ;;
    esac

    # Get download URL from GitHub API
    local api_url="https://api.github.com/repos/${REPO}/releases/latest"
    local release_info
    release_info=$(fetch "$api_url") || die "Failed to fetch release info. Install manually: ${RELEASES_URL}"

    local url
    url=$(echo "$release_info" | grep -o "\"browser_download_url\":[[:space:]]*\"[^\"]*${asset}\"" | sed 's/.*"\(http[^"]*\)".*/\1/')
    [[ -n "$url" ]] || die "No binary found for ${os}-${arch}. Install manually: ${RELEASES_URL}"

    # Download standalone binary directly to destination
    log "Downloading ${asset}..."
    download "$url" "$BINARY_PATH" || die "Download failed"
    chmod +x "$BINARY_PATH"

    # Record installed version
    local ver; ver=$(plugin_version)
    [[ "$ver" != "unknown" ]] && echo "$ver" > "$VERSION_FILE"

    log "Installed ${ver}"
}

# --- CLI ---

while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            echo "Usage: $0 [-f|--force] [-q|--quiet] [-h|--help]"
            echo "Downloads tmux-backup binary. Auto-updates when plugin version changes."
            exit 0 ;;
        -f|--force) FORCE=true; shift ;;
        -q|--quiet) QUIET=true; shift ;;
        *) die "Unknown option: $1" ;;
    esac
done

$FORCE && rm -f "$BINARY_PATH" "$VERSION_FILE"

install
