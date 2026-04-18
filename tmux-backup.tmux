#!/usr/bin/env bash

# This scripts provides a default configuration for tmux-backup options and
# key bindings. It is run only once at tmux launch.
#
# IMPORTANT: DO NOT MODIFY THIS FILE DIRECTLY!
# If you're using TPM, your changes will be lost when the plugin is updated.
#
# Instead, customize the plugin by adding options to your ~/.tmux.conf:
#
#   set -g @backup-keytable "foobar"
#   set -g @backup-keyswitch "z"
#   set -g @backup-strategy "-s most-recent -n 10"
#
# and custom bindings like:
#
#   bind-key -T foobar l 'tmux-backup catalog list'
#
# Please avoid modifying this script as it may break the integration with
# `tmux-backup`.
#
# You can also entirely ignore this file (not even source it) and define all
# options and bindings in your `tmux.conf`.

# Get the current directory where this script is located
CURRENT_DIR="$( cd "$( dirname "$0" )" && pwd )"

# Function to ensure binary is available and up to date
ensure_binary_available() {
    local installer_script="${CURRENT_DIR}/install-binary.sh"

    # Always run the installer — it checks version and skips if up to date
    if [[ -x "$installer_script" ]]; then
        if "$installer_script" --quiet; then
            return 0
        else
            echo "[$(date '+%Y-%m-%d %H:%M:%S')] tmux-backup: Warning: Failed to install binary automatically" >&2
            return 1
        fi
    else
        echo "[$(date '+%Y-%m-%d %H:%M:%S')] tmux-backup: Warning: Installer script not found" >&2
        return 1
    fi
}

# Set BINARY variable - prefer system PATH, fall back to local binary
BINARY=$(which tmux-backup 2>/dev/null || echo "")

# If not found in PATH, try to ensure local binary is available (install if needed)
if [[ -z "$BINARY" || ! -x "$BINARY" ]]; then
    ensure_binary_available

    # After ensuring local binary, check if local binary exists
    if [[ -x "${CURRENT_DIR}/tmux-backup" ]]; then
        BINARY="${CURRENT_DIR}/tmux-backup"
    fi
fi

# Check if we have a usable binary
if [[ -z "$BINARY" || ! -x "$BINARY" ]]; then
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] tmux-backup: Error: tmux-backup binary not found." >&2
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] tmux-backup: Please install manually or ensure it's in your PATH." >&2
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] tmux-backup: Visit https://github.com/graelo/tmux-backup/releases for manual download." >&2
    exit 1
fi


#
# Top-level options
#

setup_option () {
    opt_name=$1
    default_value=$2
    current_value=$(tmux show-option -gqv "@backup-${opt_name}")
    value="${current_value:-$default_value}"
    tmux set-option -g "@backup-${opt_name}" "${value}"
}


# Sets the keytable for all bindings, providing a default if @backup-keytable
# was not defined. Keytables open a new shortcut space: if 'b' is the switcher
# (see below), prefix + b + <your-shortcut>
setup_option "keytable" "tmuxbackup"

# Sets the key to access the keytable: prefix + <key> + <your-shortcut>
# providing a default if @backup-keyswitch is not defined.
setup_option "keyswitch" "b"

keyswitch=$(tmux show-option -gv @backup-keyswitch)
keytable=$(tmux show-option -gv @backup-keytable)
tmux bind-key "${keyswitch}" switch-client -T "${keytable}"

setup_option "strategy" "-s most-recent -n 10"
strategy=$(tmux show-option -gv @backup-strategy)

#
# Pattern bindings
#

setup_binding () {
    key=$1
    command="$2"
    tmux bind-key -T "${keytable}" "${key}" run-shell "${BINARY} ${command}"
}

setup_binding_w_popup () {
    key=$1
    command="$2"
    tmux bind-key -T "${keytable}" "${key}" display-popup -E "tmux new-session -A -s tmux-backup '${BINARY} ${command} ; echo Press any key... && read -k1 -s'"
}

# prefix + b + b only saves a new backup without compacting the catalog
setup_binding "b" "save ${strategy} --ignore-last-lines 1 --to-tmux"
# prefix + b + s saves a new backup and compacts the catalog
setup_binding "s" "save ${strategy} --ignore-last-lines 1 --compact --to-tmux"
# prefix + b + r restores the most recent backup
setup_binding "r" "restore ${strategy} --to-tmux"
# prefix + b + l prints the catalog without details
setup_binding_w_popup "l" "catalog ${strategy} list"
# prefix + b + L prints the catalog
setup_binding_w_popup "L" "catalog ${strategy} list --details"
