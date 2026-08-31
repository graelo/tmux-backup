# Contributing

## Build, test, check

The `Makefile` is the canonical definition of every local task; run
`make help` to list them. The ones you need day to day:

```sh
cargo build                # debug build
make release               # release build with native CPU opts
make test                  # full test suite
make check                 # fmt + lint + test — run before `git push`
make check-all             # adds audits, commit lint, docs — before a PR
make fix                   # auto-format and apply clippy fixes
```

To run a single test, use nextest directly:

```sh
cargo nextest run test_name
cargo nextest run module::tests
```

## Code coverage

```sh
make coverage
```

Report output: `./target/llvm-cov/html/index.html`

## Manpage

The manpage lives in `man/tmux-backup.1` as roff source.

Preview it with:

```sh
mandoc man/tmux-backup.1 | less
```

Lint it with `make man`.

Update the manpage when adding, removing, or renaming a CLI command or flag,
changing a default value, or changing the default tmux bindings or
`@backup-*` options. The version and date in the `.TH` header should be updated
on each release.

## Submitting Changes

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.
