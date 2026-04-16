# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.5.16] - 2026-04-17

### Changed

- Linux release binaries are now statically linked against musl
  (`x86_64-unknown-linux-musl`, `aarch64-unknown-linux-musl`), so they run on
  old distros without glibc version constraints
- Linux archives are flattened to unpack a bare `tmux-backup` binary, matching
  the macOS layout and sibling projects
- Release artifacts retention shortened to 1 day

### Fixed

- Renovate vulnerability alerts are now enabled
- Renovate `gitAuthor` matches the GitHub App bot identity
- Correct pinned-version comments for `actions/cache` and
  `bump-homebrew-formula-action`

## [0.5.15] - 2026-04-13

### Security

- Replace long-lived PATs with short-lived GitHub App tokens for release
  automation (Homebrew tap bump, Renovate)

## [0.5.14] - 2026-04-12

### Security

- Add SLSA build provenance attestation to release artifacts
- Skip the cargo cache on pushes to `main` to prevent cache poisoning

## [0.5.13] - 2026-04-11

### Changed

- Bump to Rust edition 2024 and MSRV 1.88
- Switch dependency updates from Dependabot to Renovate (runs Fridays)
- Bump dependencies

### Security

- Harden GitHub Actions workflows: pin third-party actions to commit SHAs,
  scope per-job permissions with least privilege, move secrets from action
  inputs to `env` blocks, and scope release/renovate secrets to dedicated
  GitHub Environments
- Add zizmor and poutine for workflow and CI/CD supply-chain static analysis,
  extracted into reusable workflows
- Remove cache from release workflow to prevent cache poisoning

### Fixed

- Clippy lint on Rust 1.88 (`const_is_empty` assertion in tests)
- Use `gh api` / `GH_TOKEN` for GitHub API calls (no implicit PAT usage)

## [0.5.12] - 2026-03-23

### Fixed

- Release workflow tweaks; re-release of 0.5.11 with no functional changes

## [0.5.11] - 2026-03-23

### Changed

- Drop `cargo-audit` from CI (coverage overlaps with `cargo-pants`)
- Bump `bump-homebrew-formula-action` to v4
- Bump dependencies

## [0.5.10] - 2025-12-20

### Added

- Test coverage for window/pane logic

### Fixed

- More robust window creation during restore

## [0.5.9] - 2025-12-19

### Changed

- Bump MSRV to 1.78
- Drop the Windows CI build
- Bump common GitHub Actions (`actions/checkout`, `actions/cache`)
- Bump dependencies

## [0.5.8] - 2025-11-23

### Changed

- **Breaking (internal)**: migrate from `async-std` to `smol`; upgrade to
  `tmux-lib` 0.4
- Improve `essentials` and `large-scope` CI workflows
- Update `deny.toml`

### Fixed

- Unelide a lifetime flagged by newer clippy

## [0.5.7] - 2024-08-14

### Fixed

- Release workflow: drop deprecated `::set-output`; tighten convco config

## [0.5.6] - 2024-08-14

### Fixed

- Upgrade release-related actions; drop remaining deprecated `::set-output`
- Correct `.convco` config

## [0.5.5] - 2024-08-14

### Added

- `.convco` config so the release changelog follows project conventions

### Changed

- Bump GitHub Actions versions
- Bump dependencies

## [0.5.4] - 2023-08-29

### Changed

- Bump MSRV to 1.70
- Bump dependencies

### Fixed

- Archive filename timestamps now carry microseconds only (narrower format,
  no collision edge cases)

## [0.5.3] - 2023-05-15

### Changed

- Bump dependencies

### Fixed

- Align `catalog` column names

## [0.5.2] - 2023-04-02

### Changed

- Bump MSRV to 1.64
- Switch CI to `dtolnay/rust-toolchain`
- Bump dependencies

## [0.5.1] - 2022-12-10

### Changed

- Clean up `Cargo.toml`
- Bump dependencies

### Fixed

- Clippy warnings surfaced on nightly

## [0.5.0] - 2022-11-11

### Changed

- **Breaking**: backup filenames now include a microsecond-precision timestamp
- Update docs to reflect the longer timestamp
- Bump dependencies

## [0.4.1] - 2022-11-08

### Changed

- Extract `tmux-lib` into a separate crate
- Use more idiomatic clap 4 attributes
- Cleaner, shorter CI workflow structure; don't run `cargo-outdated`/pants on
  non-Linux; download the `cargo-audit` binary directly
- Bump dependencies

## [0.4.0] - 2022-11-05

### Added

- `init` subcommand (renamed from `generate-tmux-plugin-config`) —
  **breaking rename**
- Catalog dirpath display strips `$HOME` when present

### Changed

- Documentation updates (MSRV 1.60 noted)

### Fixed

- Translate the tmux config hook to bash

## [0.3.1] - 2022-10-30

### Added

- `generate-tmux-plugin-config` subcommand to bootstrap tmux integration

### Changed

- Reorder config elements for clarity
- Split tmux buffer cleanup from capture (internal refactor)
- Docs and `tmux-backup.tmux` script updated
- Bump dependencies

## [0.3.0] - 2022-10-26

### Added

- Configurable number of trailing empty lines dropped on save

### Changed

- **Breaking**: bump MSRV to 1.60 (clap v4 requirement)
- **Breaking**: config layout — strategies moved; drop the short option for
  the classic strategy
- Pane capture trims lines and inserts a final ANSI reset code
- zsh panes no longer capture the trailing prompt line
- Inline format args; satisfy clippy pedantic
- Docs and `tmux-backup.tmux` script updated

## [0.2.0] - 2022-09-21

### Changed

- **Breaking**: archive metadata format is now JSON
- Bump dependencies

### Fixed

- Don't attempt to fetch new metadata when restoring outside of tmux

## [0.1.1] - 2022-09-10

### Added

- `tmux-backup.tmux` plugin script for direct tmux integration
- Installation docs

### Fixed

- Release changelog now includes hidden sections

## [0.1.0] - 2022-09-09

### Changed

- Use `nom` throughout tmux-layout parsing; describe nom errors
- Bump dependencies

### Fixed

- Docs: add Rust Edition badge; clean up CI (drop 1.59 matrix entry, silence
  unused steps)

## [0.0.1] - 2022-08-26

Initial public release.

### Added

- Concurrent async capture of tmux sessions, windows, and panes
- Archive saving with zstd-compressed pane content, JSON metadata, and an
  archive format version
- Restore of sessions, windows, pane layouts (parsed with `nom`), and pane
  content including ANSI escape codes
- `catalog` subcommand: `list`, `describe`, `compact` with a classic
  compaction strategy; display columns include filesize and backup age
- Shell completion via `clap_complete`; dirpath completion via `value_hint`
- `save --compact` option
- Strategy configuration via environment variables
- `show_options()` and `session_path` capture through tmux

[Unreleased]: https://github.com/graelo/tmux-backup/compare/v0.5.16...HEAD
[0.5.16]: https://github.com/graelo/tmux-backup/compare/v0.5.15...v0.5.16
[0.5.15]: https://github.com/graelo/tmux-backup/compare/v0.5.14...v0.5.15
[0.5.14]: https://github.com/graelo/tmux-backup/compare/v0.5.13...v0.5.14
[0.5.13]: https://github.com/graelo/tmux-backup/compare/v0.5.12...v0.5.13
[0.5.12]: https://github.com/graelo/tmux-backup/compare/v0.5.11...v0.5.12
[0.5.11]: https://github.com/graelo/tmux-backup/compare/v0.5.10...v0.5.11
[0.5.10]: https://github.com/graelo/tmux-backup/compare/v0.5.9...v0.5.10
[0.5.9]: https://github.com/graelo/tmux-backup/compare/v0.5.8...v0.5.9
[0.5.8]: https://github.com/graelo/tmux-backup/compare/v0.5.7...v0.5.8
[0.5.7]: https://github.com/graelo/tmux-backup/compare/v0.5.6...v0.5.7
[0.5.6]: https://github.com/graelo/tmux-backup/compare/v0.5.5...v0.5.6
[0.5.5]: https://github.com/graelo/tmux-backup/compare/v0.5.4...v0.5.5
[0.5.4]: https://github.com/graelo/tmux-backup/compare/v0.5.3...v0.5.4
[0.5.3]: https://github.com/graelo/tmux-backup/compare/v0.5.2...v0.5.3
[0.5.2]: https://github.com/graelo/tmux-backup/compare/v0.5.1...v0.5.2
[0.5.1]: https://github.com/graelo/tmux-backup/compare/v0.5.0...v0.5.1
[0.5.0]: https://github.com/graelo/tmux-backup/compare/v0.4.1...v0.5.0
[0.4.1]: https://github.com/graelo/tmux-backup/compare/v0.4.0...v0.4.1
[0.4.0]: https://github.com/graelo/tmux-backup/compare/v0.3.1...v0.4.0
[0.3.1]: https://github.com/graelo/tmux-backup/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/graelo/tmux-backup/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/graelo/tmux-backup/compare/v0.1.1...v0.2.0
[0.1.1]: https://github.com/graelo/tmux-backup/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/graelo/tmux-backup/compare/v0.0.1...v0.1.0
[0.0.1]: https://github.com/graelo/tmux-backup/releases/tag/v0.0.1
