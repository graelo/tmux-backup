# `tmux-backup`

[![crate](https://img.shields.io/crates/v/tmux-backup.svg)](https://crates.io/crates/tmux-backup)
[![documentation](https://docs.rs/tmux-backup/badge.svg)](https://docs.rs/tmux-backup)
[![minimum rustc 1.74](https://img.shields.io/badge/rustc-1.74+-red.svg)](https://rust-lang.github.io/rfcs/2495-min-rust-version.html)
[![rust 2021 edition](https://img.shields.io/badge/edition-2021-blue.svg)](https://doc.rust-lang.org/edition-guide/rust-2021/index.html)
[![build status](https://github.com/graelo/tmux-backup/actions/workflows/essentials.yml/badge.svg)](https://github.com/graelo/tmux-backup/actions/workflows/essentials.yml)

<!-- cargo-sync-readme start -->

A backup & restore solution for Tmux sessions.

Version requirement: _rustc 1.74+_

## Features

- Backup and restore of your tmux environment:
  - tmux sessions windows, panes, with layout, titles & pane history
  - current and last session.
- Fast: less than 1 sec for 16 sessions, 45 windows and 80 panes.
- Show the catalog of backups, with age, file size, content description & archive format
- 2 strategies are available:
  - keep the `n` most recent backups
  - classic backup strategy:
    - the lastest backup per hour for the past 24 hours (max 23 backups - exclude the past hour),
    - the lastest backup per day for the past 7 days (max 6 backups - exclude the past 24 hours),
    - the lastest backup per week of the past 4 weeks (max 3 backups - exclude the past week),
    - the lastest backup per month of this year (max 11 backups - exclude the past month).
- Because you decide where backups are stored, you can use both strategies, combining the
benefits of high-frequency backups and on demand backups like in tmux-resurrect.

## Getting started

After installation (see below), you can either use it from the command line, or via tmux
bindings.

The catalog is located by default in `$XDG_STATE_HOME/tmux-backup/`, or
`§HOME/.state/tmux-backup` otherwise. The default strategy is "most-recent", but you can change
it with `--strategy classic`. Check usage with `tmux-backup --help` for detailed help.

### View the catalog of existing backups

```console
$ tmux-backup catalog list --details
Strategy: KeepMostRecent: 10
Location: `$HOME/.local/state/tmux-backup`

     NAME                             AGE         STATUS       FILESIZE    VERSION  CONTENT
 11. backup-20220907T224553.156103.tar.zst   2 days      purgeable    644.17 kB   1.0      16 sessions 43 windows 79 panes
 10. backup-20220907T224926.103771.tar.zst   2 days      retainable   644.38 kB   1.0      16 sessions 43 windows 79 panes
  9. backup-20220908T092341.125258.tar.zst   2 days      retainable   654.76 kB   1.0      16 sessions 43 windows 79 panes
  8. backup-20220909T224742.781818.tar.zst   18 hours    retainable   599.64 kB   1.0      16 sessions 42 windows 77 panes
  7. backup-20220909T225158.305403.tar.zst   18 hours    retainable   600.32 kB   1.0      16 sessions 42 windows 79 panes
  6. backup-20220910T152551.807672.tar.zst   1 hour      retainable   608.79 kB   1.0      16 sessions 43 windows 80 panes
  5. backup-20220910T165118.250800.tar.zst   29 minutes  retainable   614.16 kB   1.0      16 sessions 43 windows 80 panes
  4. backup-20220910T171812.893389.tar.zst   2 minutes   retainable   614.33 kB   1.0      16 sessions 43 windows 80 panes
  3. backup-20220910T172016.924711.tar.zst   11 seconds  retainable   614.44 kB   1.0      16 sessions 43 windows 80 panes
  2. backup-20220910T172019.320809.tar.zst   8 seconds   retainable   614.42 kB   1.0      16 sessions 43 windows 80 panes
  1. backup-20220910T172024.141993.tar.zst   3 seconds   retainable   614.38 kB   1.0      16 sessions 43 windows 80 panes

11 backups: 10 retainable, 1 purgeable
```

If you installed the plugin config into tmux, then the default tmux bindings for listing
backups are

- `prefix + b + l` to show the simple catalog
- `prefix + b + L` to show the detailed catalog (adds the filesize, version & content columns)

Both of these bindings will open a tmux popup showing the catalog content.

### Save the current tmux environment

```console
$ tmux-backup save
✅ 16 sessions 43 windows 80 panes, persisted to `/Users/graelo/.local/state/tmux-backup/backup-20220910T171812.893389.tar.zst`
```

By default, the tmux binding for saving a new backup are

- `prefix + b + s` save and compact (delete purgeable backups)
- `prefix + b + b` save but not compact the catalog

Both of these bindings will print the same report as above in the tmux status bar.

### Restore from a backup

Typing `tmux-backup restore` in your shell outside of tmux will

- start a tmux server if none is running
- restore all sessions from the latest backup
- but you still have to `tmux attach -t <your-last-session>`

The same command typed in a shell inside tmux will erase session `0` (the default start
session) and restore your tmux environment in place.

By default, the tmux binding for restoring the latest backup is

- `prefix + b + r` restore sessions from the latest backup

## Installation

### Installing the binary

On macOS

```shell
brew install graelo/homebrew-tap/tmux-backup  # will also install shell completions
```

On linux

```shell
curl \
    https://github.com/graelo/tmux-backup/releases/download/v0.4.0/tmux-backup-x86_64-unknown-linux-gnu.tar.xz \
    | tar xf - > /usr/local/bin/tmux-backup
chmod +x /usr/local/bin/tmux-backup
```

On linux, to install completions, type

```shell
tmux-backup generate-completion zsh|bash|fish > /path/to/your/completions/folder
```

### Installing the tmux plugin hook

Type

```shell
mkdir ~/.tmux/plugins/tmux-backup
tmux-backup init > ~/.tmux/plugins/tmux-backup/tmux-backup.tmux
```

If you don't use [tpm](https://github.com/tmux-plugins/tpm), just add this to your
`.tmux.conf`:

```text
source-file ~/.tmux/plugins/tmux-backup/tmux-backup.tmux
```

Alternatively, if you use tpm, declare the tmux-backup plugin to TPM in your `~/.tmux.conf`:

```tmux
set -g @tpm_plugins '              \
  tmux-plugins/tpm                 \
  tmux-plugins/tmux-backup         \  <-- here
  tmux-plugins/tmux-copyrat        \
  tmux-plugins/tmux-yank           \
  tmux-plugins/tmux-resurrect      \
  tmux-plugins/tmux-sessionist     \
'
```

The next time you start tmux, the `tmux-backup.tmux` configuration will be loaded.

## Caveats

- This is a beta version
- Does not handle multiple clients: help is welcome if you have clear scenarios for this.
- Does not handle session groups: help is also welcome.

## License

Licensed under either of

- [Apache License, Version 2.0](http://www.apache.org/licenses/LICENSE-2.0)
- [MIT license](http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.

<!-- cargo-sync-readme end -->
