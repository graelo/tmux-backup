#!/usr/bin/env zsh

# This scripts provides a default configuration for tmux-backup options and
# key bindings. It is run only once at tmux launch.
#
# Each option and binding can be overridden in your `tmux.conf` by defining
# options like
#
#   set -g @backup-keytable "foobar"
#   set -g @backup-keyswitch "z"
#   set -g @backup-strategy "-s most-recent -n 10"
#
# and bindings like
#
#   bind-key -T foobar l 'tmux-backup catalog list'
#
# You can also entirely ignore this file (not even source it) and define all
# options and bindings in your `tmux.conf`.

BINARY="$(which tmux-backup)"
# CURRENT_DIR="$( cd "$( dirname "$0" )" && pwd )"
# BINARY=${CURRENT_DIR}/tmux-backup


#
# Top-level options
#


function setup_option() {
    local opt_name=$1
    local default_value=$2
    local current_value=$(tmux show-option -gqv @backup-${opt_name})
    local value=$([[ ! -z "${current_value}" ]] && echo "${current_value}" || echo "${default_value}")
    tmux set-option -g @backup-${opt_name} ${value}
}


# # Sets the window name which copyrat should use when running, providing a
# # default value in case @copyrat-window-name was not defined.
# setup_option "window-name" "[copyrat]"

# # Get that window name as a local variable for use in pattern bindings below.
# window_name=$(tmux show-option -gqv @copyrat-window-name)

# Sets the keytable for all bindings, providing a default if @backup-keytable
# was not defined. Keytables open a new shortcut space: if 't' is the switcher
# (see below), prefix + t + <your-shortcut>
setup_option "keytable" "tmuxbackup"

# Sets the key to access the keytable: prefix + <key> + <your-shortcut>
# providing a default if @backup-keyswitch is not defined.
setup_option "keyswitch" "b"

keyswitch=$(tmux show-option -gv @backup-keyswitch)
keytable=$(tmux show-option -gv @backup-keytable)
tmux bind-key ${keyswitch} switch-client -T ${keytable}

setup_option "strategy" "-s most-recent -n 10"
strategy=$(tmux show-option -gv @backup-strategy)

#
# Pattern bindings
#

function setup_binding() {
    local key=$1
    local command="$2"
    tmux bind-key -T ${keytable} ${key} run-shell "${BINARY} ${command}"
}

function setup_binding_w_popup() {
    local key=$1
    local command="$2"
    tmux bind-key -T ${keytable} ${key} display-popup -E "tmux new-session -A -s tmux-backup '${BINARY} ${command} ; echo Press any key... && read -k1 -s'"
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
