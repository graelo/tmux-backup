# AGENTS.md

This file contains instructions for coding agents working in this repository.

- Repository: <https://github.com/graelo/tmux-backup>
- Prefer `gh` for GitHub operations.
- Do not mention an agent or assistant in issues, pull requests, comments, or
  commit messages.
- Do not expose private local information, including machine-specific paths.

## Project

`tmux-backup` saves and restores tmux sessions, windows, pane layouts, and pane
history. It is a command-line program and tmux plugin, with one binary:

- `tmux-backup`: provides `save`, `autosave`, `restore`, `catalog`, `describe`,
  `generate-completion`, and `init` subcommands. `init` prints the bundled
  tmux plugin configuration.

Rust 1.95 or later is required. The crate uses edition 2024.

## Architecture

1. `src/bin/tmux-backup.rs` parses `config::Config`, dispatches the selected
   command, creates a `Catalog`, and chooses terminal or tmux-status output.
2. `src/actions/` captures state into archives, restores it, and creates the
   atomic rolling autosave. Autosaves started outside tmux select the most
   recently active attached client.
3. `src/management/` reads the backup directory, applies retention strategies,
   and owns the archive format and metadata.
4. `tmux-backup.tmux` locates or installs the binary, configures `@backup-*`
   options, and defines the default tmux bindings.

Key modules:

- `src/config.rs`: Clap command, argument, and retention-strategy definitions.
- `src/actions/save.rs`: captures sessions, windows, panes, and pane history.
- `src/actions/autosave.rs`: creates `autosave.tar.zst` atomically and handles
  scheduler-safe tmux client selection and reporting.
- `src/actions/restore.rs`: recreates sessions, windows, layouts, and pane
  content from an archive.
- `src/management/catalog.rs`: catalog discovery, listing, compaction, and
  selection of the newest ordinary backup or autosave for restoration.
- `src/management/compaction.rs`: most-recent and classic retention plans.
- `src/management/archive/v1.rs`: versioned tar+zstd archive layout,
  serialization, and metadata operations. Preserve backwards compatibility
  when changing it.
- `tmux-backup.tmux`: default plugin bindings and `@backup-*` option surface.
  It is compiled into the binary with `include_str!` and printed by
  `tmux-backup init`; changes ship with the crate.
- `install-binary.sh`: TPM-facing installer for standalone release binaries.
  Keep its asset-name logic in sync with `.github/workflows/release.yml`.

## Verification

The `Makefile` is the canonical definition of local verification tasks. **Read
it before choosing or running verification commands**; do not duplicate its
command implementations here. `make help` lists every target.

The primary targets are:

- `make check`: pre-push gate (formatting, linting, and tests).
- `make check-all`: pre-PR gate (adds dependency, commit-message, Markdown,
  and GitHub Actions security checks).
- `make fix`: formats code and applies Clippy fixes.
- `make md`: lints Markdown against `rumdl.toml`. Run it after editing any
  Markdown file; the 80-column `MD013` reflow and aligned-table `MD060` rules
  apply.
- `make ci-security`: runs the Poutine and Zizmor GitHub Actions scans.
- `make coverage`: generates an HTML report at
  `target/llvm-cov/html/index.html`.

The check targets mirror the GitHub workflows and use locked dependency
resolution where applicable. They assume their external tools (for example
`cargo-nextest`, `cargo-deny`, `cargo-pants`, `convco`, `poutine`, `zizmor`,
`rumdl`, and `cargo-llvm-cov`) are already installed locally.

For focused Rust tests, use `cargo nextest run <test_name>` or
`cargo nextest run <module::tests::name>`. The complete CI test sequence is
implemented in `ci/test_full.sh`; its Nextest CI profile is configured in
`.config/nextest.toml`.

## Documentation and releases

Keep user-facing documentation in sync with behavior:

- Update `README.md` for CLI behavior, installation, plugin bindings, and
  release artifact changes. Update `AUTOSAVE.md` when scheduler/autosave
  behavior changes.
- Update `tmux-backup.tmux` when changing default bindings or `@backup-*`
  options. It implements the configuration printed by `tmux-backup init`, so a
  documentation-only change does not alter the installed plugin.
- Update `install-binary.sh` and the release workflow together when changing
  supported release targets or asset names.
- For a release version bump, update `Cargo.toml`, `Cargo.lock`, and the
  versioned section and comparison links in `CHANGELOG.md`. Create a
  `vX.Y.Z` tag; the release workflow builds, attests, archives, and publishes
  artifacts from that tag, then updates the Homebrew formula for stable tags.
- Commit messages must follow `.convco` Conventional Commit rules. Use
  `make commits` to check them.

`Cargo.toml`, `Cargo.lock`, `deny.toml`, and the GitHub workflows define the
release and supply-chain constraints. Preserve `--locked` behavior in Cargo
commands that resolve dependencies.
