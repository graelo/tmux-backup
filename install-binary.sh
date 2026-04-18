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

    # Build expected asset name
    local asset
    if [[ "$os" == "darwin" ]]; then
        asset="${BINARY}-${arch}-apple-darwin.zip"
    else
        asset="${BINARY}-${arch}-unknown-linux-musl.tar.xz"
    fi

    # Get download URL from GitHub API
    local api_url="https://api.github.com/repos/${REPO}/releases/latest"
    local release_info
    release_info=$(fetch "$api_url") || die "Failed to fetch release info. Install manually: ${RELEASES_URL}"

    local url
    url=$(echo "$release_info" | grep -o "\"browser_download_url\":[[:space:]]*\"[^\"]*${asset}\"" | sed 's/.*"\(http[^"]*\)".*/\1/')
    [[ -n "$url" ]] || die "No binary found for ${os}-${arch}. Install manually: ${RELEASES_URL}"

    # Download and extract
    log "Downloading ${asset}..."
    local tmp="${SCRIPT_DIR}/${asset}"
    download "$url" "$tmp" || die "Download failed"

    if [[ "$asset" == *.zip ]]; then
        unzip -qo "$tmp" -d "$SCRIPT_DIR"
    else
        tar -xf "$tmp" -C "$SCRIPT_DIR"
    fi
    rm -f "$tmp"

    # Find and move binary to expected location
    local found=""
    for candidate in "${SCRIPT_DIR}/${BINARY}" "${SCRIPT_DIR}/target/release/${BINARY}" "${SCRIPT_DIR}/release/${BINARY}"; do
        [[ -f "$candidate" ]] && found="$candidate" && break
    done
    [[ -n "$found" ]] || die "Binary not found after extraction"

    if [[ "$found" != "$BINARY_PATH" ]]; then
        mv "$found" "$BINARY_PATH"
        # Clean up leftover empty directories
        local d; d=$(dirname "$found")
        while [[ "$d" != "$SCRIPT_DIR" ]] && [[ -d "$d" ]]; do
            rmdir "$d" 2>/dev/null || break
            d=$(dirname "$d")
        done
    fi
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
